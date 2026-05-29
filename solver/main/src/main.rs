use std::io::Write;

use flams_math_archives::{
    Archive,
    backend::{AnyBackend, GlobalBackend, LocalBackend},
    source_files::SourceEntry,
    utils::AllSyncEngine,
};
use ftml_ontology::{
    domain::modules::{Module, ModuleLike},
    terms::Term,
    utils::{RefTree, time::measure},
};
use ftml_solver::{
    Checker, CheckerCache, SubtermCheckResult,
    results::DocumentCheckResult,
    split::{
        RayonSplit, RayonStrategiesDepth, RayonStrategiesOnly, SingleThreadedSplit, SplitStrategy,
    },
};
use ftml_uris::{ArchiveId, DocumentUri};

flams_math_archives::source_format!(STEX {
    name: "stex",
    description: "(Semantically annotated) LaTeX",
    targets: &[],
    file_extensions: &["tex", "ltx"],
    dependencies: |_| Vec::new()
});

fn main() {
    let _ = enable_ansi_support::enable_ansi_support();
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    GlobalBackend::initialize::<AllSyncEngine>();
    /*let _ = std::thread::Builder::new()
    .stack_size(6 * 1024 * 1024)
    .spawn(move || {*/
    //pause();
    let (i, t) = measure(check_subterm); //(check_selected); //measure(check_all); //
    println!("Checked {i} documents in {t}");
    /*println!(
        "minimal stack: {}",
        bytesize::ByteSize::b(minimal_stack() as _)
            .display()
            .iec_short()
    );*/
    /*    })
    .expect("wut")
    .join();*/
}

fn check_subterm() -> usize {
    const TOP: &str = r#"{"Application":{"head":{"Symbol":{"uri":"http://mathhub.info?a=FTML/math&p=propositions&m=equivalence&s=equivalence","presentation":null}},"arguments":[{"Simple":{"Application":{"head":{"Symbol":{"uri":"http://mathhub.info?a=Papers/26-Intelligencer-sTeX&p=mod&m=subset&s=fuzzy subset","presentation":null}},"arguments":[{"Simple":{"Var":{"variable":{"Ref":{"declaration":"http://mathhub.info?a=Papers/26-Intelligencer-sTeX&p=mod&d=subset&l=en&e=vA","is_sequence":false}},"presentation":null}}},{"Simple":{"Var":{"variable":{"Ref":{"declaration":"http://mathhub.info?a=Papers/26-Intelligencer-sTeX&p=mod&d=subset&l=en&e=vB","is_sequence":false}},"presentation":null}}}],"presentation":null}}},{"Simple":{"Application":{"head":{"Symbol":{"uri":"http://mathhub.info?a=Papers/26-Intelligencer-sTeX&p=mod&m=lift&s=less than or equal to","presentation":null}},"arguments":[{"Sequence":{"Seq":[{"Application":{"head":{"Symbol":{"uri":"http://mathhub.info?a=Papers/26-Intelligencer-sTeX&p=mod&m=fuzzyset&s=membership function","presentation":null}},"arguments":[{"Simple":{"Var":{"variable":{"Ref":{"declaration":"http://mathhub.info?a=Papers/26-Intelligencer-sTeX&p=mod&d=subset&l=en&e=vA","is_sequence":false}},"presentation":null}}}],"presentation":null}},{"Application":{"head":{"Symbol":{"uri":"http://mathhub.info?a=Papers/26-Intelligencer-sTeX&p=mod&m=fuzzyset&s=membership function","presentation":null}},"arguments":[{"Simple":{"Var":{"variable":{"Ref":{"declaration":"http://mathhub.info?a=Papers/26-Intelligencer-sTeX&p=mod&d=subset&l=en&e=vB","is_sequence":false}},"presentation":null}}}],"presentation":null}}]}}],"presentation":null}}}],"presentation":null}}"#;
    const SUB: &str = r#"{"Application":{"head":{"Symbol":{"uri":"http://mathhub.info?a=Papers/26-Intelligencer-sTeX&p=mod&m=subset&s=fuzzy subset","presentation":null}},"arguments":[{"Simple":{"Var":{"variable":{"Ref":{"declaration":"http://mathhub.info?a=Papers/26-Intelligencer-sTeX&p=mod&d=subset&l=en&e=vA","is_sequence":null}},"presentation":null}}},{"Simple":{"Var":{"variable":{"Ref":{"declaration":"http://mathhub.info?a=Papers/26-Intelligencer-sTeX&p=mod&d=subset&l=en&e=vB","is_sequence":null}},"presentation":null}}}],"presentation":null}}"#;
    let top: Term = serde_json::from_str(TOP).unwrap();
    let sub: Term = serde_json::from_str(SUB).unwrap();
    let mut global_context = rustc_hash::FxHashSet::default();
    for m in top.full_context(&mut |u| AnyBackend::Global.get_document(u).ok()) {
        global_context.insert(m);
    }
    let mut checker =
        Checker::<SingleThreadedSplit /*RayonStrategiesDepth<4>*/>::new(AnyBackend::Global);
    checker.set_context(global_context.into_iter().collect());
    let SubtermCheckResult {
        simplified,
        inferred_type,
        context,
        log,
    } = checker.check_subterm_term(top, sub).expect("failed");

    println!("{}", log.colored());
    println!(
        "{:?}\n  : {:?}",
        simplified.debug_short(),
        inferred_type.as_ref().map(Term::debug_short)
    );
    1
}

fn check_selected() -> usize {
    thread_local! {
        static CACHE: std::cell::Cell<CheckerCache> = std::cell::Cell::new(CheckerCache::default());
    }
    //for _ in 0..1 {
    macro_rules! check {
            ($($s:literal),* $(,)?) => {
                {
                    let mut i = 0;
                    $(
                        i += 1;
                        let mut solver = Checker::<SingleThreadedSplit/*RayonStrategiesDepth<4>*/>::new(AnyBackend::Global);
                        solver.set_cache(CACHE.take());
                        check(&mut solver,$s);
                        CACHE.set(solver.into_cache());
                    )*
                    i
                }
            }
        }
    check!(
        /*
        "http://mathhub.info?a=FTML/math&p=sets&d=comprehension&l=en",
        "http://mathhub.info?a=FTML/math&p=proofs&d=axiom&l=en",
        "http://mathhub.info?a=FTML/math&p=sets&d=cons&l=en",
        "http://mathhub.info?a=FTML/math&p=sets&d=inset&l=en",
        "http://mathhub.info?a=FTML/math&p=sets&d=cartesian-product&l=en",
        "http://mathhub.info?a=FTML/math&p=nat&d=nat&l=en",
        "http://mathhub.info?a=FTML/math&p=propositions&d=prop&l=en",
        "http://mathhub.info?a=FTML/math&p=propositions&d=negation&l=en",
        "http://mathhub.info?a=FTML/math&p=propositions&d=conjunction&l=en",
        "http://mathhub.info?a=FTML/math&p=propositions&d=disjunction&l=en",
        "http://mathhub.info?a=FTML/math&p=propositions&d=implication&l=en",
        "http://mathhub.info?a=FTML/math&p=propositions&d=equivalence&l=en",
        "http://mathhub.info?a=FTML/math&p=propositions&d=forall&l=en",
        "http://mathhub.info?a=FTML/math&p=propositions&d=exists&l=en",
        "http://mathhub.info?a=FTML/math&p=propositions&d=equal&l=en",
        "http://mathhub.info?a=FTML/math&d=functions&l=en",
        "http://mathhub.info?a=FTML/math&p=proofs&d=judgment&l=en",
        "http://mathhub.info?a=FTML/math&p=proofs&d=inference-rule&l=en",
        "http://mathhub.info?a=FTML/math&p=proofs/natural-deduction&d=implication-introduction&l=en",
        "http://mathhub.info?a=FTML/math&p=proofs/natural-deduction&d=conjunction-introduction&l=en",
        "http://mathhub.info?a=FTML/math&p=proofs/natural-deduction&d=conjunction-elimination&l=en",
        "http://mathhub.info?a=FTML/math&p=proofs/natural-deduction&d=exists-elimination&l=en",
        "http://mathhub.info?a=FTML/math&p=proofs&d=choice-operator&l=en",
        "http://mathhub.info?a=FTML/math&p=arithmetics&d=sqrt&l=en",
        "http://mathhub.info?a=unimarx/werkbank&p=sec/einstimmungundgrundbegriffe/mod&d=evaluationmap&l=de",
        "http://mathhub.info?a=FTML/tests&d=othertests&l=en",
        "http://mathhub.info?a=FTML/tests&d=sqrt2&l=en",
        "http://mathhub.info?a=FTML/tests&d=probdists&l=en",
        */
        "http://mathhub.info?a=FTML/math&p=numbers/real&d=order&l=en",
        //"http://mathhub.info?a=Papers/26-ICMS-Semantics&p=fragments&d=variables&l=en",
        //"http://mathhub.info?a=FTML/tests&d=units&l=en",
        //"http://mathhub.info?a=FTML/tests&d=natded&l=en",
    )
    //}
}

fn check_all() -> usize {
    static PATH: &str = "/home/jazzpirate/work/Software/FlexiFormal/FLAMS/solver/foo.txt";
    const SAVE: bool = false;
    const LOAD: bool = false;
    let read_file = if LOAD {
        std::fs::read_to_string(PATH)
            .expect("möp")
            .split("\n\n%%%%%%%%%%\n\n")
            .filter_map(|s| {
                if s.trim().is_empty() {
                    None
                } else {
                    Some(s.trim().to_string())
                }
            })
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    let mut out_file = if SAVE {
        Some(std::io::BufWriter::new(
            std::fs::File::create(PATH).expect("möp"),
        ))
    } else {
        None
    };
    let alldocs = all_documents(|a| a.as_ref().starts_with("FTML/math"));
    let mut dones = 0;
    let mut failures = 0;

    for d in alldocs {
        let mut solver =
            Checker::</*SingleThreadedSplit */ RayonStrategiesDepth<4>>::new(AnyBackend::Global);
        let Ok(d) = GlobalBackend.get_document(&d) else {
            continue;
        };
        println!("{}", d.uri);
        let ((mut v, _), t) = measure(|| solver.check_document(&d).expect("dependency missing"));
        let fails = count_fails(&v);
        failures += fails;
        if fails != 0 {
            v.filter_failures();
            let vs = v.display::<()>().to_string();
            if let Some(f) = out_file.as_mut() {
                let _ = f.write_all(format!("{vs}\n\n%%%%%%%%%%\n\n").as_bytes());
            }
            if !read_file.contains(&vs) {
                println!("{}", v.colored());
            }
        }
        println!("Checked after {t}");
        dones += 1;
    }
    println!("Total failures: {failures}");
    dones
}

fn count_fails(res: &DocumentCheckResult) -> usize {
    res.checks.iter().filter(|c| !c.success()).count()
}

fn pause() {
    use std::io::{Read, Write};
    let mut stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    write!(
        stdout,
        r"
        --------------------------------------------------------------------------
        Press any key to continue..

    "
    )
    .expect("wut");
    let _ = stdout.flush();
    let _ = stdin.read(&mut [0u8]);
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
}

#[allow(clippy::needless_collect)]
pub fn all_documents(filter: fn(&ArchiveId) -> bool) -> Vec<DocumentUri> {
    use flams_math_archives::MathArchive;
    let backend = flams_math_archives::backend::GlobalBackend.get();
    let uris = backend
        .all_archives()
        .iter()
        .filter_map(|a| {
            let Archive::Local(a) = a else { return None };
            if !filter(a.id()) {
                return None;
            }
            Some(a.with_sources(|src| {
                src.dfs()
                    .filter_map(|d| {
                        let SourceEntry::File(f) = d else { return None };
                        Some(a.source_dir().join(f.relative_path.as_ref()))
                    })
                    .collect::<Vec<_>>()
            }))
        })
        .flatten()
        .collect::<Vec<_>>();
    let uris = uris
        .into_iter()
        .filter_map(|f| backend.uri_of(&f))
        .collect::<Vec<_>>();
    tracing::info!("{} documents", uris.len());
    uris
}

fn check<Split: SplitStrategy>(solver: &mut Checker<Split>, s: &str) {
    println!("Checking {s}");
    let d = GlobalBackend
        .get_document(&s.parse().expect("uri wut"))
        .expect("wut");
    let ((mut v, _), t) = measure(|| solver.check_document(&d).expect("dependency missing"));
    let failures = count_fails(&v);

    //v.filter_failures();
    println!("{}", v.colored());
    println!("Checked after {t}");

    /*
    if failures == 0 {
        println!("Checked after {t}");
    } else {
        //v.filter_failures();
        println!("{}", v.colored());
        println!("Checked after {t}");
        pause();
    }
    */
}

fn get_module(s: &str) -> Module {
    let ModuleLike::Module(m) = GlobalBackend
        .get()
        .get_module(&s.parse().expect("wut"))
        .expect("wut")
    else {
        panic!("wut")
    };
    m
}
