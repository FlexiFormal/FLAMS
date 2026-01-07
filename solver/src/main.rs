use flams_math_archives::{
    backend::{AnyBackend, GlobalBackend, LocalBackend},
    utils::AllSyncEngine,
};
use ftml_ontology::{
    domain::modules::{Module, ModuleLike},
    utils::time::measure,
};
use solver::{
    Checker,
    split::{RayonSplit, RayonStrategiesOnly, SingleThreadedSplit, SplitStrategy},
};

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
        //.with_max_level(tracing::Level::TRACE)
        .init();
    GlobalBackend::initialize::<AllSyncEngine>();

    //pause();
    let (i, t) = measure(move || {
        //for _ in 0..1 {
        let mut solver = Checker::<SingleThreadedSplit>::new(AnyBackend::Global);
        macro_rules! check {
                ($($s:literal),* $(,)?) => {
                    {
                        let mut i = 0;
                        $(
                            i += 1;
                            check(&mut solver,$s);
                        )*
                        i
                    }
                }
            }
        check!(
            /*
            "http://mathhub.info?a=FTML/math&p=propositions&d=prop&l=en",
            "http://mathhub.info?a=FTML/math&p=propositions&d=negation&l=en",
            "http://mathhub.info?a=FTML/math&p=propositions&d=conjunction&l=en",
            "http://mathhub.info?a=FTML/math&p=propositions&d=disjunction&l=en",
            "http://mathhub.info?a=FTML/math&p=propositions&d=implication&l=en",
            "http://mathhub.info?a=FTML/math&p=propositions&d=equivalence&l=en",
            "http://mathhub.info?a=FTML/math&p=sets&d=inset&l=en",
            "http://mathhub.info?a=FTML/math&p=propositions&d=forall&l=en",
            "http://mathhub.info?a=FTML/math&p=propositions&d=exists&l=en",
            "http://mathhub.info?a=FTML/math&p=propositions&d=equal&l=en",
            "http://mathhub.info?a=FTML/math&d=functions&l=en",
            */
            "http://mathhub.info?a=FTML/math&p=sets&d=cons&l=en",
            /*
            "http://mathhub.info?a=FTML/math&p=sets&d=comprehension&l=en",
            "http://mathhub.info?a=FTML/math&p=nat&d=nat&l=en",
            "http://mathhub.info?a=FTML/math&d=test&l=en",
            */
        )
        //}
    });
    println!("Checked {i} documents in {t}");
    /*println!(
        "minimal stack: {}",
        bytesize::ByteSize::b(minimal_stack() as _)
            .display()
            .iec_short()
    );*/
}

fn pause() {
    use std::io::{Read, Write};
    let mut stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    write!(stdout, "Press any key to continue...").expect("wut");
    let _ = stdout.flush();
    let _ = stdin.read(&mut [0u8]);
}

fn check<Split: SplitStrategy>(solver: &mut Checker<Split>, s: &str) {
    println!("Checking {s}");
    let d = GlobalBackend
        .get_document(&s.parse().expect("uri wut"))
        .expect("wut");
    let (v, t) = measure(|| solver.check_document(&d));
    for v in v {
        println!("{}", v.display().colorize_stdout());
    }
    println!("Checked after {t}");
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
