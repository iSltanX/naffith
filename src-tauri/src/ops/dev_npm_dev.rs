//! خادم تطوير مشروع Node.js، باستخدام `npm run dev`.
//!
//! ## الفرق الحقيقي عن أخواتها الأربع في هذا القسم
//!
//! `typecheck`/`lint`/`test`/`install` تنتهي من تلقاء نفسها. هذه **لا
//! تنتهي أبدًا** ما لم يُلغِها المستخدم — خادم تطويرٍ (‏vite، webpack، …)
//! يبقى يستمع ويعيد البناء عند كل تعديل حتى يُوقَف يدويًا. وهذا ليس فرقًا في
//! درجة الخطورة (`Danger`)، بل في **نمط التشغيل**: لا رقم مهلةٍ هنا، ولا
//! سقفٌ زمني — `executor.rs` ينتظر خروج العملية أو إلغاء المستخدم أيّهما
//! أسبق، بلا فرعٍ ثالث. انظر `run_inner` في ذلك الملف.
//!
//! والإلغاء هنا **هو الطريق الطبيعي الوحيد للإنهاء**، لا استثناءً: `Cancelled`
//! دلالةٌ ناجحة محايدة في هذا التطبيق أصلًا (`ResultSemantic::Cancelled`)،
//! فلا حاجة إلى معنًى جديد — فقط نصٌّ صادق في `i18n.ts` يقول «شغّال حتى
//! توقفه» لا «سينتهي قريبًا».
//!
//! ولا ناتج مؤقّت هنا يحتاج تنظيفًا عند الإلغاء (`Conflict::NoArtifact`)،
//! لكن `setsid`/`killpg` في `executor.rs` يضمنان أن الإلغاء يطال شجرة
//! العمليات كاملة — الخادم وكل ما فرّعه (‏esbuild، عمّال vite، …) — لا الأب
//! وحده.

use crate::error::Result;
use crate::ops::dev_common::npm_run;
use crate::spec::*;
use crate::value::Inputs;

pub const ID: &str = "dev.npm.dev";

pub const SPEC: OperationSpec = OperationSpec {
    id: ID,
    title_key: "op.dev.npm.dev.title",
    description_key: "op.dev.npm.dev.description",
    category: Category::Developer,
    danger: Danger::Safe,
    visibility: Visibility::Production,
    tool: crate::tools::NPM,
    conflict: Conflict::NoArtifact,
    inputs: &[
        InputSpec::new("project", InputKind::ExistingDir),
        InputSpec::new("node_path", InputKind::ExistingFile),
    ],
    sort_order: 50,
    search_terms: &["dev", "npm", "node", "خادم", "تطوير", "vite", "webpack", "مشروع"],
    plan,
};

fn plan(inputs: &Inputs) -> Result<PlannedCommand> {
    let project = inputs.dir("project")?;
    let node_path = inputs.file("node_path")?;
    npm_run(node_path, project, "dev").reveal(project).read_only()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::Scratch;
    use crate::value::RawValue;
    use std::collections::BTreeMap;
    use std::path::{Path, PathBuf};

    fn node_stub(s: &Scratch) -> PathBuf {
        let bin = s.dir("bin");
        for name in ["node", "npm"] {
            let p = bin.join(name);
            std::fs::write(&p, b"#!/bin/sh\nexit 0\n").unwrap();
            let mut perm = std::fs::metadata(&p).unwrap().permissions();
            std::os::unix::fs::PermissionsExt::set_mode(&mut perm, 0o755);
            std::fs::set_permissions(&p, perm).unwrap();
        }
        bin.join("node")
    }

    fn plan_with(project: &Path, node: &Path) -> Result<PlannedCommand> {
        let raw = BTreeMap::from([
            ("project".to_owned(), RawValue::Path(project.display().to_string())),
            ("node_path".to_owned(), RawValue::Path(node.display().to_string())),
        ]);
        plan(&crate::value::validate(&SPEC, &raw)?)
    }

    #[test]
    fn the_operation_is_listed_in_its_category() {
        let listed = crate::registry::list(crate::policy::Policy::production());
        let found = listed.iter().find(|o| o.id == ID).expect("must be listed");
        assert_eq!(found.category, Category::Developer);
        assert_eq!(found.conflict, Conflict::NoArtifact);
    }

    #[test]
    fn the_argv_runs_npm_run_dev() {
        let s = Scratch::new("npm-dev").unwrap();
        let project = s.dir("مشروع");
        let node = node_stub(&s);
        let cmd = plan_with(&project, &node).unwrap();
        let args: Vec<String> = cmd.args.iter().map(|a| a.to_string_lossy().into_owned()).collect();
        assert_eq!(args, vec!["run", "dev"]);
    }

    /// أهمّ ما يُثبت هنا: العملية **لا تصرّح بأي حدٍّ زمني**. `PlannedCommand`
    /// لا يملك حقل مهلة أصلًا — والاختبار الحقيقي أن هذا الملف لا يحاول
    /// إضافة واحد، فيبقى انتظار الخروج في `executor.rs` بلا سقفٍ كما صُمِّم.
    #[test]
    fn planning_declares_no_timeout_the_executor_already_waits_indefinitely() {
        let s = Scratch::new("npm-dev-no-timeout").unwrap();
        let project = s.dir("مشروع");
        let node = node_stub(&s);
        let cmd = plan_with(&project, &node).unwrap();
        // لا حقل `timeout`/`deadline` على `PlannedCommand` — هذا الاختبار
        // يوثّق القرار لا يفحص نوعًا: أي إضافةٍ لحقل كهذا يجب أن تُراجَع هنا
        // أولًا، لأنها تكسر بالضبط ما تحتاجه هذه العملية.
        let _ = cmd; // تأكيد أن البناء نجح بلا أي مهلة، لا أكثر.
    }
}
