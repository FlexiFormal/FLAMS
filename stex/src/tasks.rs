use async_trait::async_trait;
use immt_api::formats::building::{Backend, BuildData, BuildResult, TaskStep};
use std::process::Stdio;

#[derive(Copy, Clone)]
pub struct PdfLaTeX;

impl TaskStep for PdfLaTeX {
    fn run(&self, state: &mut BuildData, backend: &Backend<'_>) -> BuildResult {
        let file = state.build_file().unwrap();
        let out = match std::process::Command::new("pdflatex")
            .args(["-interaction", "nonstopmode", "-halt-on-error"])
            .arg(file.file_stem().unwrap())
            .current_dir(file.parent().unwrap())
            .env("STEX_USESMS", "false")
            .env("MATHHUB", backend.mathhub)
            .stdout(Stdio::piped())
            .spawn()
        {
            Ok(c) => c.wait_with_output(),
            Err(_) => return BuildResult::Err("Failed to run pdflatex".into()),
        };
        match out {
            Err(e) => BuildResult::Err(format!("pdflatex failed: {}", e).into()),
            Ok(o) => {
                if !o.status.success() {
                    let out = std::str::from_utf8(o.stdout.as_slice()).unwrap();
                    let err = out
                        .find("Fatal error")
                        .map(|i| &out[i..])
                        .unwrap_or("unknown error");
                    BuildResult::Err(format!("pdflatex failed: {}", err).into())
                } else {
                    let bib = file.with_extension("bcf");
                    if bib.exists() {
                        std::process::Command::new("biber")
                            .arg(file.file_stem().unwrap())
                            .current_dir(file.parent().unwrap())
                            .stdout(Stdio::piped())
                            .spawn()
                            .unwrap()
                            .wait()
                            .unwrap();
                    } else {
                        std::process::Command::new("bibtex")
                            .arg(file.file_stem().unwrap())
                            .current_dir(file.parent().unwrap())
                            .stdout(Stdio::piped())
                            .spawn()
                            .unwrap()
                            .wait()
                            .unwrap();
                    }
                    BuildResult::Success
                }
            }
        }
    }
}

#[derive(Copy, Clone)]
pub struct RusTeX;
#[async_trait]
impl TaskStep for RusTeX {
    fn run(&self, state: &mut BuildData, backend: &Backend<'_>) -> BuildResult {
        // Do Something
        BuildResult::Success
    }
}
