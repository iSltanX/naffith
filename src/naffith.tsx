/**
 * نَفِّذ — السطح الهادئ.
 *
 * النموذج يُشتقّ من `OperationSummary` القادم من الفهرس، لا من قائمة حقول
 * مكتوبة هنا: العملية تعلن مدخلاتها، والشاشة ترسمها. وكل تغيير يعيد التخطيط
 * فورًا، فالمعاينة والملخّص وشاشة «سَطْر» كلها ناتج نفس الخطة الحيّة.
 */
import { useEffect, useId, useRef } from 'react';
import type { CoreErrorShape, PlanResponse } from './ipc';
import { asCoreError, pickDirectory } from './ipc';
import { errorText, t } from './i18n';

type Phase = 'idle' | 'planning' | 'running' | 'cancelling' | 'finished';

export interface FormValues {
  source: string;
  destination: string;
  archive_name: string;
}

interface Props {
  values: FormValues;
  onChange: (next: FormValues) => void;
  plan: PlanResponse | null;
  error: CoreErrorShape | null;
  phase: Phase;
  outcome: { status: string; produced?: string | null; key?: string } | null;
  onExecute: () => void;
  onCancel: () => void;
  onReveal: () => void;
  onReset: () => void;
  opTitleKey: string;
  opDescriptionKey: string;
}

/** حقل مسار: يُملأ بالحوار أو بالكتابة. الاثنان يمرّان بنفس التحقّق. */
function PathField({
  id,
  label,
  help,
  placeholder,
  value,
  error,
  disabled,
  onPick,
  onType,
}: {
  id: string;
  label: string;
  help: string;
  placeholder: string;
  value: string;
  error: string | null;
  disabled: boolean;
  onPick: () => void;
  onType: (v: string) => void;
}) {
  const errorId = `${id}-error`;
  const helpId = `${id}-help`;
  return (
    <div className="row-field">
      <label className="t-label" htmlFor={id}>
        {label}
      </label>
      <div className="row-field__control">
        {/* المسار LTR داخل نموذج عربي: قراءته من اليمين تفسد ترتيب مقاطعه. */}
        <span className="field field--path">
          <svg viewBox="0 0 24 24" aria-hidden="true" className="field__icon">
            <use href="#i-folder" />
          </svg>
          <input
            id={id}
            type="text"
            dir="ltr"
            spellCheck={false}
            autoComplete="off"
            value={value}
            placeholder={placeholder}
            disabled={disabled}
            aria-describedby={error ? `${errorId} ${helpId}` : helpId}
            aria-invalid={error ? true : undefined}
            onChange={(e) => onType(e.target.value)}
          />
        </span>
        <button type="button" className="btn btn--quiet" onClick={onPick} disabled={disabled}>
          <svg viewBox="0 0 24 24" aria-hidden="true">
            <use href="#i-folder-open" />
          </svg>
          {value ? t('field.chosen') : t('field.choose')}
        </button>
      </div>
      <p id={helpId} className="t-helper">
        {help}
      </p>
      {error && (
        <p id={errorId} className="field-error" role="alert">
          <svg viewBox="0 0 24 24" aria-hidden="true">
            <use href="#i-error" />
          </svg>
          {error}
        </p>
      )}
    </div>
  );
}

export default function Naffith(props: Props) {
  const {
    values,
    onChange,
    plan,
    error,
    phase,
    outcome,
    onExecute,
    onCancel,
    onReveal,
    onReset,
    opTitleKey,
    opDescriptionKey,
  } = props;

  const uid = useId();
  const nameId = `${uid}-name`;
  const nameErrorId = `${nameId}-error`;
  const firstField = useRef<HTMLInputElement>(null);

  const busy = phase === 'running' || phase === 'cancelling';
  const errorFor = (field: string): string | null =>
    error && error.input === field ? errorText(error.key, error.detail) : null;
  // خطأ لا ينسبه النواةُ إلى حقل يُعرض فوق الأزرار لا تحت حقل عشوائي.
  const generalError = error && !error.input ? errorText(error.key, error.detail) : null;

  useEffect(() => {
    if (phase === 'idle' && !values.source) firstField.current?.focus();
  }, [phase, values.source]);

  async function pick(field: 'source' | 'destination') {
    try {
      const chosen = await pickDirectory();
      if (chosen) onChange({ ...values, [field]: chosen });
    } catch {
      // فشل الحوار لا يُعطّل الحقل: الكتابة واللصق يبقيان طريقًا كاملًا.
    }
  }

  return (
    <section className="naffith card card--naffith" aria-labelledby="naffith-heading">
      <header className="naffith__head">
        <div>
          <h2 id="naffith-heading" className="t-section-title">
            {t('app.naffith')}
          </h2>
          <p className="t-card-title naffith__op">{t(opTitleKey)}</p>
        </div>
        <span className="chip chip--info">
          <svg viewBox="0 0 24 24" aria-hidden="true">
            <use href="#i-plus" />
          </svg>
          {t('summary.danger.creates')}
        </span>
      </header>

      <p className="t-body-sec naffith__desc">{t(opDescriptionKey)}</p>

      <div className="naffith__form">
        <PathField
          id={`${uid}-source`}
          label={t('field.source.label')}
          help={t('field.source.help')}
          placeholder={t('field.source.placeholder')}
          value={values.source}
          error={errorFor('source')}
          disabled={busy}
          onPick={() => pick('source')}
          onType={(v) => onChange({ ...values, source: v })}
        />

        <PathField
          id={`${uid}-destination`}
          label={t('field.destination.label')}
          help={t('field.destination.help')}
          placeholder={t('field.destination.placeholder')}
          value={values.destination}
          error={errorFor('destination')}
          disabled={busy}
          onPick={() => pick('destination')}
          onType={(v) => onChange({ ...values, destination: v })}
        />

        <div className="row-field">
          <label className="t-label" htmlFor={nameId}>
            {t('field.archive_name.label')}
          </label>
          <div className="row-field__control">
            {/* اسم الملف نصّ المستخدم، فيبقى باتجاه الصفحة لا LTR قسرًا. */}
            <span className="field">
              <svg viewBox="0 0 24 24" aria-hidden="true" className="field__icon">
                <use href="#i-compress" />
              </svg>
              <input
                id={nameId}
                ref={firstField}
                type="text"
                spellCheck={false}
                autoComplete="off"
                value={values.archive_name}
                placeholder={t('field.archive_name.placeholder')}
                disabled={busy}
                aria-invalid={errorFor('archive_name') ? true : undefined}
                aria-describedby={errorFor('archive_name') ? nameErrorId : undefined}
                onChange={(e) => onChange({ ...values, archive_name: e.target.value })}
              />
            </span>
            <span className="suffix-hint lat" aria-hidden="true">
              .zip
            </span>
          </div>
          <p className="t-helper">{t('field.archive_name.help')}</p>
          {errorFor('archive_name') && (
            <p id={nameErrorId} className="field-error" role="alert">
              <svg viewBox="0 0 24 24" aria-hidden="true">
                <use href="#i-error" />
              </svg>
              {errorFor('archive_name')}
            </p>
          )}
        </div>
      </div>

      <Summary plan={plan} phase={phase} />

      {generalError && (
        <p className="field-error field-error--block" role="alert">
          <svg viewBox="0 0 24 24" aria-hidden="true">
            <use href="#i-error" />
          </svg>
          {generalError}
        </p>
      )}

      <div className="naffith__actions">
        {phase !== 'finished' && (
          <>
            <button
              type="button"
              className="btn btn--primary btn--lg"
              onClick={onExecute}
              disabled={!plan || busy || phase === 'planning'}
            >
              <svg viewBox="0 0 24 24" aria-hidden="true">
                <use href="#i-execute" />
              </svg>
              {t('action.execute')}
            </button>
            {busy && (
              <button
                type="button"
                className="btn btn--danger"
                onClick={onCancel}
                disabled={phase === 'cancelling'}
              >
                <svg viewBox="0 0 24 24" aria-hidden="true">
                  <use href="#i-close" />
                </svg>
                {t('action.cancel')}
              </button>
            )}
          </>
        )}
        {phase === 'finished' && (
          <button type="button" className="btn btn--quiet btn--lg" onClick={onReset}>
            {t('action.again')}
          </button>
        )}
      </div>

      <RunState phase={phase} outcome={outcome} onReveal={onReveal} />
    </section>
  );
}

/**
 * حجم مقروء بالعربية.
 *
 * القاعدة العشرية (١٠٠٠) لا الثنائية: الرقم يُقارَن بمساحة قرص، وFinder نفسه
 * يعرض القرص عشريًا. عرضُ ٩٥٤ م.ب لما يسمّيه النظام غيغابايت ارتباك لا دقّة.
 * والرقم يبقى LTR داخل الجملة العربية عبر `<bdi>` في موضع الاستدعاء.
 */
export function readableSize(bytes: number): string {
  const units: [number, string][] = [
    [1e9, 'unit.gb'],
    [1e6, 'unit.mb'],
    [1e3, 'unit.kb'],
  ];
  for (const [scale, key] of units) {
    if (bytes >= scale) {
      const n = bytes / scale;
      // منزلة واحدة تحت ١٠، وعدد صحيح فوقها: «١٢٫٤ م.ب» مفيد، «١٢٤٫٧ م.ب» ضجيج.
      return `${n < 10 ? n.toFixed(1) : Math.round(n)} ${t(key)}`;
    }
  }
  return `${bytes} ${t('unit.bytes')}`;
}

/** ملخّص بلغة المستخدم: ماذا يُنشأ، من أين، وما الذي لن يحدث. */
function Summary({ plan, phase }: { plan: PlanResponse | null; phase: Phase }) {
  if (phase === 'finished') return null;
  if (!plan) {
    return (
      <div className="summary summary--empty">
        <p className="t-body-sec">
          {phase === 'planning' ? t('state.checking') : t('summary.incomplete')}
        </p>
      </div>
    );
  }

  return (
    <div className="summary" aria-live="polite">
      <h3 className="t-label summary__title">{t('summary.title')}</h3>

      <p className="summary__line">
        <span className="t-body-sec">{t('summary.will_create')}</span>
        <bdi className="path summary__path">{plan.produces}</bdi>
      </p>
      <p className="summary__line">
        <span className="t-body-sec">{t('summary.from')}</span>
        <bdi className="path summary__path">{plan.argv_display[plan.argv_display.length - 2]}</bdi>
      </p>

      {/* الأداة والتقدير: حقائق قبل التنفيذ، لا بعده. */}
      {/* `dt`/`dd` أبناءٌ مباشرون للشبكة لا مغلَّفون، وإلا صار كل صفّ شبكةً
          مستقلّة فاختلف عرض عموده الأول وتعرّجت القيم. */}
      <dl className="summary__meta">
        <dt className="t-body-sec">{t('summary.tool')}</dt>
        <dd>
          <code className="lat">{plan.tool.id}</code>{' '}
          <bdi className="path summary__path">{plan.tool.path}</bdi>
        </dd>

        {plan.estimate && (
          <>
            <dt className="t-body-sec">{t('summary.estimate')}</dt>
            <dd>
              {/* عزلٌ بلا فرض اتجاه: العبارة تخلط رقمًا لاتينيًا بوحدة عربية،
                  فإجبارها LTR كان يقلبها إلى «م.ب ١٢ ≈». «≈» ليست زينة — هي
                  وسم التقدير نفسه، ومكانها صدر العبارة. */}
              <bdi>≈ {readableSize(plan.estimate.approx_source_bytes)}</bdi>
              <span className="t-caption summary__estimate-note">
                {' '}
                {t(
                  plan.estimate.complete ? 'summary.estimate.note' : 'summary.estimate.partial',
                )}
              </span>
            </dd>
          </>
        )}
      </dl>

      <ul className="summary__facts">
        <li>{t('summary.untouched')}</li>
        {/* سياسة التضارب تُقرأ من الخطة لا تُكتب هنا: نصٌّ ثابت كان سيبقى
            معروضًا لو تغيّرت السياسة يومًا، فيَعِد بما لا تفعله النواة. */}
        <li>{t(`summary.conflict.${plan.conflict}`)}</li>
        <li>{t('summary.atomic')}</li>
      </ul>

      {plan.warnings.length > 0 && (
        <ul className="summary__warnings">
          {plan.warnings.map((w) => (
            <li key={w} className="warning">
              <svg viewBox="0 0 24 24" aria-hidden="true">
                <use href="#i-warning" />
              </svg>
              <span>{t(w)}</span>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}

/**
 * حالة التشغيل.
 *
 * لا شريط تقدّم ولا نسبة مئوية: `ditto` لا تعطي تقدّمًا موثوقًا، ورقمٌ مخترع
 * يزحف إلى ٩٩٪ ثم يقف أسوأ من غياب الرقم. ما يُعرض دوّار وحالة عمل، مع سبب
 * غيابه مكتوبًا.
 */
function RunState({
  phase,
  outcome,
  onReveal,
}: {
  phase: Phase;
  outcome: { status: string; produced?: string | null; key?: string } | null;
  onReveal: () => void;
}) {
  if (phase === 'running' || phase === 'cancelling') {
    return (
      <div className="runstate runstate--busy" role="status">
        <span className="spinner" aria-hidden="true" />
        <div>
          <p className="t-body">
            {phase === 'cancelling' ? t('state.cancelling') : t('state.running')}
          </p>
          <p className="t-caption">{t('state.running.note')}</p>
        </div>
      </div>
    );
  }

  if (phase !== 'finished' || !outcome) return null;

  if (outcome.status === 'success') {
    return (
      <div className="runstate runstate--ok" role="status">
        <span className="chip chip--success">
          <svg viewBox="0 0 24 24" aria-hidden="true">
            <use href="#i-success" />
          </svg>
          {t('state.succeeded')}
        </span>
        <bdi className="path runstate__path">{outcome.produced}</bdi>
        <button type="button" className="btn btn--quiet" onClick={onReveal}>
          <svg viewBox="0 0 24 24" aria-hidden="true">
            <use href="#i-external" />
          </svg>
          {t('action.reveal')}
        </button>
      </div>
    );
  }

  if (outcome.status === 'cancelled') {
    return (
      <div className="runstate runstate--neutral" role="status">
        <span className="chip chip--neutral">
          <svg viewBox="0 0 24 24" aria-hidden="true">
            <use href="#i-cancelled" />
          </svg>
          {t('state.cancelled')}
        </span>
        <p className="t-body-sec">{t('state.cancelled.note')}</p>
      </div>
    );
  }

  return (
    <div className="runstate runstate--bad" role="alert">
      <span className="chip chip--danger">
        <svg viewBox="0 0 24 24" aria-hidden="true">
          <use href="#i-error" />
        </svg>
        {t('state.failed')}
      </span>
      <p className="t-body-sec">
        {outcome.key ? errorText(outcome.key) : t('state.failed.note')}
      </p>
      {outcome.key && <p className="t-caption">{t('state.failed.note')}</p>}
    </div>
  );
}

export { asCoreError };
