// @vitest-environment jsdom
/**
 * التوليف نفسه — لا الوصل بالتشغيل. `app.test.tsx` يحرس **متى** تُستدعى
 * `playSuccessChime` (عند النجاح، والإعداد مفعَّل)؛ وهذا يحرس أنها **لا
 * ترمي أبدًا** مهما كانت بيئة التشغيل، وأنها تصنع نغمةً فعلًا حين تملك
 * `AudioContext`.
 *
 * ‏`class` حقيقية لا `vi.fn(() => ...)`: الكود المُختبَر يستدعي المُنشئ
 * بـ`new`، ودالّة سهميّةٌ ملفوفة بـ`vi.fn` لا تدعم ذلك — فتفشل صامتًا (يمسكها
 * `try/catch` في `playSuccessChime` نفسه) وتبدو النغمة وكأنها لم تُصنع قط.
 */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { playSuccessChime } from './notification-sound';

class MockOscillator {
  type = '';
  frequency = { value: 0 };
  connect = vi.fn();
  start = vi.fn();
  stop = vi.fn();
}

class MockGainParam {
  setValueAtTime = vi.fn();
  linearRampToValueAtTime = vi.fn();
  exponentialRampToValueAtTime = vi.fn();
}

class MockGain {
  gain = new MockGainParam();
  connect = vi.fn();
}

class MockAudioContext {
  currentTime = 0;
  destination = { id: 'destination' };
  createOscillator = vi.fn(() => new MockOscillator());
  createGain = vi.fn(() => new MockGain());
  close = vi.fn(() => Promise.resolve());
}

afterEach(() => {
  vi.unstubAllGlobals();
  vi.useRealTimers();
});

describe('حين لا AudioContext في البيئة', () => {
  it('لا ترمي — بيئة اختبارٍ عادية بلا الواجهة', () => {
    // jsdom لا يعرّف AudioContext افتراضيًا؛ هذا هو الحال الطبيعي هنا بلا
    // أي استهزاء إضافي.
    expect(() => playSuccessChime()).not.toThrow();
  });
});

describe('حين تملك البيئة AudioContext', () => {
  let instances: MockAudioContext[];
  let ConstructorSpy: typeof MockAudioContext;

  beforeEach(() => {
    instances = [];
    // ‏`class` تمتدّ الأصلية فتبقى قابلةً لـ`new` تمامًا كأصلها، وتسجّل كل
    // نسخةٍ تُبنى — بديلٌ عن `vi.fn()` الذي لا يدعم البناء بدالّةٍ سهمية.
    ConstructorSpy = class extends MockAudioContext {
      constructor() {
        super();
        instances.push(this);
      }
    };
    vi.stubGlobal('AudioContext', ConstructorSpy);
  });

  it('تصنع نغمتين متصلتين بالوجهة، بلا رمي', () => {
    expect(() => playSuccessChime()).not.toThrow();

    expect(instances).toHaveLength(1);
    const context = instances[0] as MockAudioContext;
    expect(context.createOscillator).toHaveBeenCalledTimes(2);
    expect(context.createGain).toHaveBeenCalledTimes(2);

    const oscillators = context.createOscillator.mock.results.map((r) => r.value as MockOscillator);
    const gains = context.createGain.mock.results.map((r) => r.value as MockGain);

    for (const osc of oscillators) {
      expect(osc.type).toBe('sine');
      expect(osc.frequency.value).toBeGreaterThan(0);
      expect(osc.start).toHaveBeenCalledTimes(1);
      expect(osc.stop).toHaveBeenCalledTimes(1);
      // النهاية بعد البداية دائمًا — نغمةٌ بمدّةٍ حقيقية لا صفرية.
      const startAt = osc.start.mock.calls[0]?.[0] as number;
      const stopAt = osc.stop.mock.calls[0]?.[0] as number;
      expect(stopAt).toBeGreaterThan(startAt);
    }

    for (const gain of gains) {
      // مغلَّفٌ يصعد من صفرٍ ثم يهبط — لا نقرة بدء ولا قطع مفاجئ.
      expect(gain.gain.setValueAtTime).toHaveBeenCalledWith(0, expect.any(Number));
      expect(gain.gain.linearRampToValueAtTime).toHaveBeenCalled();
      expect(gain.gain.exponentialRampToValueAtTime).toHaveBeenCalled();
      expect(gain.connect).toHaveBeenCalledWith(context.destination);
    }
  });

  it('النغمة الثانية أعلى نبرةً وتبدأ بعد الأولى — صعودٌ لا تكرار', () => {
    playSuccessChime();
    const context = instances[0] as MockAudioContext;
    const oscillators = context.createOscillator.mock.results.map((r) => r.value as MockOscillator);
    const [first, second] = oscillators as [MockOscillator, MockOscillator];

    expect(second.frequency.value).toBeGreaterThan(first.frequency.value);
    const firstStart = first.start.mock.calls[0]?.[0] as number;
    const secondStart = second.start.mock.calls[0]?.[0] as number;
    expect(secondStart).toBeGreaterThan(firstStart);
  });

  it('تستعمل webkitAudioContext حين AudioContext غائبة', () => {
    vi.unstubAllGlobals();
    vi.stubGlobal('webkitAudioContext', ConstructorSpy);

    playSuccessChime();

    expect(instances).toHaveLength(1);
  });

  it('تغلق السياق بعد انتهاء النغمة الأخيرة، لا فورًا', () => {
    vi.useFakeTimers();
    playSuccessChime();
    const context = instances[0] as MockAudioContext;

    expect(context.close).not.toHaveBeenCalled();
    vi.advanceTimersByTime(1000);
    expect(context.close).toHaveBeenCalledTimes(1);
  });

  it('لا ترمي حتى لو رمى بناء السياق نفسه', () => {
    class ThrowingContext {
      constructor() {
        throw new Error('AudioContext refused (autoplay policy)');
      }
    }
    vi.stubGlobal('AudioContext', ThrowingContext);

    expect(() => playSuccessChime()).not.toThrow();
  });

  it('لا ترمي حتى لو رمى إنشاء المذبذب', () => {
    class FailingContext extends MockAudioContext {
      override createOscillator = vi.fn(() => {
        throw new Error('boom');
      });
    }
    vi.stubGlobal('AudioContext', FailingContext);

    expect(() => playSuccessChime()).not.toThrow();
  });
});
