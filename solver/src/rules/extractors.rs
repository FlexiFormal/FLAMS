use crate::{rules::RuleSet, split::SplitStrategy};
use ftml_ontology::{
    domain::declarations::symbols::{AssocType, Symbol},
    terms::Term,
};
use ftml_uris::Id;

pub type SymbolRuleExtractor<Split> = fn(&Symbol, &mut RuleSet<Split>);
pub type RuleExtractor<Split> = (&'static str, fn(&[Term], &mut RuleSet<Split>));

#[must_use]
pub const fn all_symbol_extractors<Split: SplitStrategy>() -> &'static [SymbolRuleExtractor<Split>]
{
    &[
        prenex,
        binl,
        binr,
        conj,
        pwconj,
        reorder,
        of_type,
        apply,
        lambda,
        pi,
        prop,
        judgment,
        universe,
        inhabitable,
        any,
        implicit,
        conjunction,
    ]
}
#[must_use]
pub const fn all_rule_extractors<Split: SplitStrategy>() -> &'static [RuleExtractor<Split>] {
    &[("hoas-lambda-pi-apply", hoas_lpa)]
}

macro_rules! rules {
    (
        $(
            $v:vis $name:ident $( ($id:expr) )? = ($symbol:ident,$rules:ident) => $b:block
        )*
    ) => {
        $( rules!(@I $v $name $(($id))? = ($symbol,$rules) => $b ); )*
    };
    (@I $v:vis $name:ident($id:expr) = ($symbol:ident,$rules:ident) => $b:block) => {
        $v fn $name<Split: SplitStrategy>($symbol:&Symbol,$rules:&mut RuleSet<Split>) {
            static ID: std::sync::LazyLock<Id> =
                std::sync::LazyLock::new(|| unsafe { $id.parse().unwrap_unchecked() });
            if $symbol.data.role.contains(&ID) $b
        }
    };
    (@I $v:vis $name:ident = ($symbol:ident,$rules:ident) => $b:block) => {
        rules!(@I $v $name(stringify!($name)) = ($symbol,$rules) => $b);
    }
}

pub fn hoas_lpa<Split: SplitStrategy>(params: &[Term], rules: &mut RuleSet<Split>) {
    if params.len() != 3 {
        return;
    }
    let (
        Term::Symbol { uri: lambda, .. },
        Term::Symbol { uri: pi, .. },
        Term::Symbol { uri: apply, .. },
    ) = (&params[0], &params[1], &params[2])
    else {
        return;
    };
    rules.push_inhabitable(Box::new(super::pi::PiRule(pi.clone())));
    rules.push_inference(Box::new(super::pi::PiRule(pi.clone())));
    rules.push_inference(Box::new(super::pi::LambdaPiRule {
        lambda: lambda.clone(),
        pi: pi.clone(),
    }));
    rules.push_checking(Box::new(super::pi::LambdaPiRule {
        lambda: lambda.clone(),
        pi: pi.clone(),
    }));
}

pub fn reorder<Split: SplitStrategy>(sym: &Symbol, rules: &mut RuleSet<Split>) {
    if let Some(perm) = &sym.data.reordering {
        rules.push_preparation(Box::new(super::fixity::ReorderRule {
            symbol: sym.uri.clone(),
            reorder: perm.clone(),
        }));
    }
}

pub fn prenex<Split: SplitStrategy>(sym: &Symbol, rules: &mut RuleSet<Split>) {
    if sym.data.assoctype.is_some_and(|at| at == AssocType::Prenex) {
        rules.push_preparation(Box::new(super::fixity::PrenexRule(sym.uri.clone())));
    }
}

pub fn conj<Split: SplitStrategy>(sym: &Symbol, rules: &mut RuleSet<Split>) {
    if sym
        .data
        .assoctype
        .is_some_and(|at| at == AssocType::Conjunctive)
    {
        rules.push_preparation(Box::new(super::fixity::ConjunctiveRule(sym.uri.clone())));
    }
}

pub fn pwconj<Split: SplitStrategy>(sym: &Symbol, rules: &mut RuleSet<Split>) {
    if sym
        .data
        .assoctype
        .is_some_and(|at| at == AssocType::PairwiseConjunctive)
    {
        rules.push_preparation(Box::new(super::fixity::PairwiseConjunctiveRule(
            sym.uri.clone(),
        )));
    }
}

pub fn binl<Split: SplitStrategy>(sym: &Symbol, rules: &mut RuleSet<Split>) {
    if sym
        .data
        .assoctype
        .is_some_and(|at| at == AssocType::LeftAssociativeBinary)
    {
        rules.push_preparation(Box::new(super::fixity::BinLRule(sym.uri.clone())));
    }
}

pub fn binr<Split: SplitStrategy>(sym: &Symbol, rules: &mut RuleSet<Split>) {
    if sym
        .data
        .assoctype
        .is_some_and(|at| at == AssocType::RightAssociativeBinary)
    {
        rules.push_preparation(Box::new(super::fixity::BinRRule(sym.uri.clone())));
    }
}

rules! {
    pub conjunction = (sym,rules) => {
        rules.push_preparation(Box::new(super::fixity::IsConjunctionRule(sym.uri.clone())));
    }
    pub universe = (sym,rules) => {
        rules.push_inhabitable(Box::new(super::universe::SimpleUniverseRule(sym.uri.clone())));
        rules.push_universe(Box::new(super::universe::SimpleUniverseRule(sym.uri.clone())));
    }
    pub of_type("oftype") = (sym,rules) => {
        rules.push_preparation(Box::new(super::typing::SimpleTypeOperatorRule(sym.uri.clone())));
    }
    pub apply = (sym,rules) => {

    }
    pub lambda = (sym,rules) => {

    }
    pub pi = (sym,rules) => {
        rules.push_inhabitable(Box::new(super::pi::PiRule(sym.uri.clone())));
        rules.push_inference(Box::new(super::pi::PiRule(sym.uri.clone())));
    }
    pub prop = (sym,rules) => {

    }
    pub judgment = (sym,rules) => {

    }
    pub inhabitable = (sym,rules) => {
        rules.push_inhabitable(Box::new(super::universe::SimpleInhabitableRule(sym.uri.clone(),sym.data.arity.num())));
    }
    pub any = (sym,rules) => {
        rules.push_inhabitable(Box::new(super::universe::SimpleInhabitableRule(sym.uri.clone(),0)));
        rules.push_subtyping(Box::new(super::universe::AnyRule(sym.uri.clone())));
        rules.push_checking(Box::new(super::universe::AnyRule(sym.uri.clone())));
    }
    pub implicit = (sym,rules) => {

    }
}
