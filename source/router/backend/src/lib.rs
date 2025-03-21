#[cfg(any(
    all(feature = "ssr", feature = "hydrate", not(doc)),
    not(any(feature = "ssr", feature = "hydrate"))
))]
compile_error!("exactly one of the features \"ssr\" or \"hydrate\" must be enabled");
