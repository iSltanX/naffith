/**
 * سَطْر — الطبقة المكشوفة تحت السطح الهادئ.
 *
 * ثلاث قواعد تحكم هذه الشاشة:
 *
 * 1. **تعرض الأمر نفسه، لا تمثيلًا له.** كل رمز هنا يأتي من `plan.explain`،
 *    وهي مبنيّة في Rust من `argv` نفسها التي ستذهب إلى `execve`. اختبارٌ ذهبي
 *    يثبت التطابق رمزًا رمزًا.
 *
 * 2. **الوسائط مفصولة بصريًا.** أمرٌ معروض سطرًا واحدًا يجعل المسافة داخل اسم
 *    ملف تبدو كفاصل بين وسيطين. هنا كل وسيط صفّ مرقّم، فلا التباس.
 *
 * 3. **النسخ ليس تنفيذًا.** الأمر المنسوخ يمرّ بهروب صدفة لأن Terminal يحتاجه؛
 *    التطبيق لا يستعمل ذلك النص أبدًا. انظر `shell-quote.ts`.
 */
import { useState } from 'react';
import type { PlanResponse, TokenRole } from './ipc';
import { t } from './i18n';
import { shellCommand, tokenNotes } from './shell-quote';

const ROLE_CLASS: Record<TokenRole, string> = {
  tool: 'tok-name',
  flag: 'tok-flag',
  path: 'tok-path',
  value: 'tok-string',
};

const ROLE_LABEL: Record<TokenRole, string> = {
  tool: 'satr.legend.tool',
  flag: 'satr.legend.flag',
  path: 'satr.legend.path',
  value: 'satr.arg',
};

export default function Satr({ plan }: { plan: PlanResponse | null }) {
  const [copied, setCopied] = useState(false);

  async function copy() {
    if (!plan) return;
    await navigator.clipboard.writeText(shellCommand(plan.argv_display));
    setCopied(true);
    window.setTimeout(() => setCopied(false), 1600);
  }

  if (!plan) {
    return (
      <section className="satr" aria-labelledby="satr-heading">
        <div className="satr__head">
          <h2 id="satr-heading" className="t-section-title">
            {t('app.satr')}
          </h2>
        </div>
        <p className="t-body-sec satr__empty">{t('satr.empty')}</p>
      </section>
    );
  }

  // آخر وسيطين هما المصدر والمؤقّت. نعطيهما شرحًا خاصًا لأنهما بيانات لا رايات.
  const lastIndex = plan.explain.length - 1;
  const sourceIndex = lastIndex - 1;

  return (
    <section className="satr" aria-labelledby="satr-heading">
      <div className="satr__head">
        <h2 id="satr-heading" className="t-section-title">
          {t('app.satr')}
        </h2>
        <p className="t-caption satr__subtitle">{t('satr.subtitle')}</p>
      </div>

      <div className="command">
        <div className="command__bar">
          <span className="command__label">{t('satr.title')}</span>
          <button
            type="button"
            className="btn btn--quiet btn--sm"
            onClick={copy}
            aria-live="polite"
          >
            <svg viewBox="0 0 24 24" aria-hidden="true">
              <use href={copied ? '#i-check' : '#i-copy'} />
            </svg>
            {copied ? t('action.copied') : t('action.copy')}
          </button>
        </div>

        {/* سطر واحد للقراءة السريعة، بنفس ترتيب الوسائط ولونها. */}
        <div className="command__body">
          <span className="tok-prompt">$ </span>
          {plan.argv_display.map((token, i) => (
            <span key={i}>
              <span className={ROLE_CLASS[plan.explain[i]?.role ?? 'value']}>{token}</span>
              {i < plan.argv_display.length - 1 ? ' ' : ''}
            </span>
          ))}
        </div>
      </div>

      {/* الوسائط مفصولة: هنا لا يمكن أن تُقرأ مسافةٌ داخل اسمٍ فاصلًا. */}
      <ol className="args" aria-label="وسائط الأمر، كلٌّ على حدة">
        {plan.explain.map((tok, i) => {
          const notes = tokenNotes(tok.token);
          const label =
            i === sourceIndex
              ? 'explain.role.source'
              : i === lastIndex
                ? 'explain.role.temp'
                : tok.key;
          return (
            <li className="arg" key={i}>
              <span className="arg__index lat" aria-hidden="true">
                {i === 0 ? '·' : i}
              </span>
              <div className="arg__main">
                <code className={`arg__token ${ROLE_CLASS[tok.role]}`}>{tok.token}</code>
                {label && <p className="t-caption arg__note">{t(label)}</p>}
                {notes.length > 0 && (
                  <p className="t-caption arg__flagged">
                    <svg viewBox="0 0 24 24" aria-hidden="true">
                      <use href="#i-info" />
                    </svg>
                    {notes.join(' · ')}
                  </p>
                )}
              </div>
              <span className="arg__role t-caption">{t(ROLE_LABEL[tok.role])}</span>
            </li>
          );
        })}
      </ol>

      <div className="satr__notes">
        <p className="t-body-sec">{t('satr.no_shell')}</p>
        <p className="t-body-sec">{t('satr.promotion')}</p>
        <p className="t-caption satr__fineprint">{t('satr.copy_note')}</p>
      </div>
    </section>
  );
}
