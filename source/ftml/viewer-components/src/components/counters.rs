use flams_ontology::{narration::{documents::DocumentStyles, paragraphs::ParagraphKind, sections::SectionLevel}, uris::{DocumentURI, Name}};
use flams_utils::vecmap::{VecMap, VecSet};
use leptos::prelude::*;
use smallvec::SmallVec;

use crate::extractor::DOMExtractor;

use super::{TOCElem, TOCIter};

#[derive(Debug,Clone,Copy,serde::Serialize,serde::Deserialize,PartialEq,Eq)]
pub enum LogicalLevel {
    None,
    Section(SectionLevel),
    Paragraph,
    BeamerSlide
}

trait CounterTrait:Copy+PartialEq+
  std::ops::Add<Self,Output=Self>+
  std::ops::AddAssign<Self>+
  Default+Clone+Send+Sync+
  std::fmt::Debug+std::fmt::Display+'static
{
  fn one() -> Self;
}
impl CounterTrait for u16 {
  fn one() -> Self { 1 }
}
impl CounterTrait for u32 {
  fn one() -> Self { 1 }
}

#[derive(Copy,Clone,PartialEq,Eq,Default,Debug)]
struct AllSections(pub [u16;7]);
impl std::fmt::Display for AllSections {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f,"[{} {} {} {} {} {} {}]",self.0[0],self.0[1],self.0[2],self.0[3],self.0[4],self.0[5],self.0[6])
  }
}

impl std::ops::Add<SectionLevel> for AllSections {
  type Output = Self;
  fn add(self, rhs: SectionLevel) -> Self::Output {
    let idx : u8 = rhs.into();
    let mut s = AllSections::default();
    s.0[idx as usize] = 1;
    self + s
  }
}

impl std::ops::Add<Self> for AllSections {
  type Output = Self;
  fn add(self, rhs: Self) -> Self::Output {
    let mut changed = false;
    let r= AllSections([
      {if rhs.0[0]>0 {changed = true}; self.0[0]+rhs.0[0]},
      {if rhs.0[1]>0 {changed=true}; self.0[1]+rhs.0[1] },
      {if changed {0} else {if rhs.0[2]>0 {changed=true} self.0[2]+rhs.0[2] }},
      {if changed {0} else {if rhs.0[3]>0 {changed=true} self.0[3]+rhs.0[3] }},
      {if changed {0} else {if rhs.0[4]>0 {changed=true} self.0[4]+rhs.0[4] }},
      {if changed {0} else {if rhs.0[5]>0 {changed=true} self.0[5]+rhs.0[5] }},
      {if changed {0} else {if rhs.0[6]>0 {changed=true} self.0[6]+rhs.0[6] }},
    ]);
    //tracing::warn!("updating {self:?}+{rhs:?}={r:?}");
    r
  }
}

impl std::ops::AddAssign<Self> for AllSections {
  fn add_assign(&mut self, rhs: Self) {
    //tracing::warn!("updating {self:?}+{rhs:?}");
    let mut changed = false;
    if rhs.0[0] > 0 {changed=true}; self.0[0]+=rhs.0[0];
    if rhs.0[1] > 0 {changed=true;} self.0[1]+=rhs.0[1];
    if changed {self.0[2] = 0} else {if rhs.0[2] > 0 {changed=true;} self.0[2]+=rhs.0[2];}
    if changed {self.0[3] = 0} else {if rhs.0[3] > 0 {changed=true;} self.0[3]+=rhs.0[3];}
    if changed {self.0[4] = 0} else {if rhs.0[4] > 0 {changed=true;} self.0[4]+=rhs.0[4];}
    if changed {self.0[5] = 0} else {if rhs.0[5] > 0 {changed=true;} self.0[5]+=rhs.0[5];}
    if changed {self.0[6] = 0} else {if rhs.0[6] > 0 {changed=true;} self.0[6]+=rhs.0[6];}
    //tracing::warn!(" = {self:?}");
  }
}

impl CounterTrait for AllSections {
  fn one() -> Self { panic!("That's not how sectioning works") }
}

impl SmartCounter<AllSections> {
  fn inc_at(&self,lvl:SectionLevel) {
    let idx : u8 = lvl.into();
    let mut s = AllSections::default();
    s.0[idx as usize] = 1;
    self.0.update_untracked(|SmartCounterI{since,..}| *since += s );
  }
}

#[derive(Debug,Clone,Default)]
struct SmartCounterI<N:CounterTrait> {
  cutoff:Option<Cutoff<N>>,
  since:N
}

#[derive(Clone)]
struct Cutoff<N:CounterTrait> {
  previous:Option<std::sync::Arc<Cutoff<N>>>,
  since:N,
  set:RwSignal<N>
}
impl<N:CounterTrait> Cutoff<N> {
  fn get(&self) -> N {
    if let Some(p) = self.previous.as_ref() {
      //leptos::logging::log!("Getting {}+{}+{}",p.get(),self.since,self.set.get());
      p.get() + self.since + self.set.get()
    } else {
      //leptos::logging::log!("Getting {}+{}",self.since,self.set.get());
      self.since + self.set.get()
    }
  }
  fn depth(&self) -> u16 {
    if let Some(p) = self.previous.as_ref() {
      p.depth() + 1
    } else { 1 }
  }
  fn get_untracked(&self) -> N {
    if let Some(p) = self.previous.as_ref() {
      p.get_untracked() + self.since + self.set.get_untracked()
    } else {
      self.since + self.set.get_untracked()
    }
  }
}
impl<N:CounterTrait> std::fmt::Debug for Cutoff<N> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Cutoff")
      .field("depth", &self.depth())
      .field("previous", &self.previous.as_ref().map(|p| p.get_untracked()))
      .field("since", &self.since)
      .field("set", &self.set.get_untracked())
      .finish()
  }
}

#[derive(Clone,Default,Copy)]
struct SmartCounter<N:CounterTrait>(RwSignal<SmartCounterI<N>>);

impl<N:CounterTrait> SmartCounterI<N> {
  fn get(&self) -> N {
    if let Some(cutoff) = &self.cutoff {
      cutoff.get() + self.since
    } else { self.since }
  }
}

impl<N:CounterTrait> std::fmt::Debug for SmartCounter<N> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.0.with_untracked(|s| f.debug_struct("SmartCounter").field("inner", s).finish())
  }
}

impl<N:CounterTrait> SmartCounter<N> {
  fn inc_memo<T:Send+Sync+'static+PartialEq>(&self,f:impl Fn(N) -> T + Send + Sync + 'static) -> Memo<T> {
    self.0.update_untracked(|SmartCounterI{cutoff,since}| {
      *since += N::one();
      let since = *since;
      if let Some(cutoff) = cutoff {
        let cutoff = cutoff.clone();
        Memo::new(move |_| f(cutoff.get() + since))
      } else {
        Memo::new(move |_| f(since))
      }
    })
  }
  fn get_untracked(&self) -> N {
    self.0.with_untracked(|SmartCounterI{cutoff,since}| if let Some(cutoff) = cutoff {
      cutoff.get() + *since
    } else {
     *since
    })
  }

  fn inc(&self) {
    self.0.update_untracked(|SmartCounterI{since,..}| *since += N::one() );
  }
  fn memo<T:Send+Sync+'static+PartialEq>(&self,f:impl Fn(N) -> T + Send + Sync + 'static) -> Memo<T> {
    self.0.with_untracked(|SmartCounterI{cutoff,since}| {
      let since = *since;
      if let Some(cutoff) = cutoff {
        let cutoff = cutoff.clone();
        Memo::new(move |_| f(cutoff.get() + since))
      } else {
        Memo::new(move |_| f(since))
      }
    })
  }
  fn reset(&self) {
    self.0.update_untracked(|x| *x = SmartCounterI::default());
  }

  fn set_cutoff(&self,v:N) {
    self.0.update_untracked(|SmartCounterI{cutoff,..}| {
      if let Some(c) = cutoff.as_ref() {
        c.set.set(v);
      }
    });
  }

  fn split(&self) -> Self {
    let SmartCounterI{cutoff,since} = self.0.get_untracked();
    let ret = Self(RwSignal::new(SmartCounterI{cutoff:cutoff.clone(),since}));

    let previous = cutoff.map(std::sync::Arc::new);
    let new_cutoff = Cutoff {
      previous,since,set:RwSignal::new(N::default())
    };
    self.0.update_untracked(|SmartCounterI{cutoff:nctf,since:snc}| {
      *nctf = Some(new_cutoff);
      *snc = N::default();
    });
    ret
  }
}

#[derive(Debug,Clone)]
pub struct SectionCounters {
    current: LogicalLevel,
    pub max: SectionLevel,
    sections:SmartCounter<AllSections>,
    initialized:RwSignal<bool>,
    counters:RwSignal<VecMap<Name,SmartCounter<u16>>>,
    resets:RwSignal<VecMap<SectionLevel,VecSet<Name>>>,
    for_paras:RwSignal<VecMap<(ParagraphKind,Option<Name>),Option<Name>>>,
    slides:SmartCounter<u32>
}
impl Default for SectionCounters {
    #[inline]
    fn default() -> Self {
        Self { current: LogicalLevel::None, max: SectionLevel::Part, 
          sections:SmartCounter::default(),
          counters:RwSignal::new(VecMap::default()), 
          resets:RwSignal::new(VecMap::default()),
          for_paras:RwSignal::new(VecMap::default()),
          initialized:RwSignal::new(false),
          slides:SmartCounter::default()
        }
    }
}
impl SectionCounters {

  fn init_paras(&self) {
    if self.initialized.get_untracked() { return }
    self.initialized.update_untracked(|b| *b = true);
    let mut counters = VecMap::default();
    let mut resets = VecMap::default();
    let mut for_paras = VecMap::default();
    let sig = expect_context::<RwSignal<DOMExtractor>>();
    sig.with_untracked(|e| {
      for c in &e.styles.counters {
        //leptos::logging::log!("Doing {c:?}");
        counters.insert(c.name.clone(),SmartCounter::default());
        if let Some(p) = c.parent {
          resets.get_or_insert_mut(p,|| VecSet::new()).insert(c.name.clone());
        }
      }
      for stl in &e.styles.styles {
        for_paras.insert((stl.kind,stl.name.clone()),stl.counter.clone());
      }
    });
    self.counters.update_untracked(|p| *p = counters);
    self.resets.update_untracked(|p| *p = resets);
    self.for_paras.update_untracked(|p| *p = for_paras);
  }

  pub fn current_level(&self) -> LogicalLevel {
    self.current
  }

  pub fn next_section(&mut self) -> (Option<Memo<String>>,Option<&'static str>) {
    self.init_paras();
    let lvl = if let LogicalLevel::Section(s) = self.current {
      s.inc()
    } else if self.current == LogicalLevel::None { self.max } else {
      return ((Some(Memo::new(|_| "display:content;".into())),None));
    };
    //tracing::warn!("New section at level {lvl:?}");
    self.set_section(lvl);
    let sections = self.sections.0.get_untracked();
    (match lvl {
      SectionLevel::Part => (
        Some(Memo::new(move |_| {let sects = sections.get().0; 
          format!("counter-set:ftml-part {};",sects[0])})),
        Some("ftml-part")
      ),
      SectionLevel::Chapter => (
        Some(Memo::new(move |_| {let sects = sections.get().0; 
          format!("counter-set:ftml-part {} ftml-chapter {}",
          sects[0],
          sects[1]
        )})),
        Some("ftml-chapter")
      ),
      SectionLevel::Section => (
        Some(Memo::new(move |_| {let sects = sections.get().0; 
          format!("counter-set:ftml-part {} ftml-chapter {} ftml-section {}",
          sects[0],
          sects[1],
          sects[2]
        )})),
        Some("ftml-section")
      ),
      SectionLevel::Subsection => (
        Some(Memo::new(move |_| {let sects = sections.get().0; 
          format!("counter-set:ftml-part {} ftml-chapter {} ftml-section {} ftml-subsection {}",
          sects[0],
          sects[1],
          sects[2],
          sects[3],
        )})),
        Some("ftml-subsection")
      ),
      SectionLevel::Subsubsection => (
        Some(Memo::new(move |_| {let sects = sections.get().0; 
          format!("counter-set:ftml-part {} ftml-chapter {} ftml-section {} ftml-subsection {} ftml-subsubsection {}",
          sects[0],
          sects[1],
          sects[2],
          sects[3],
          sects[4],
        )})),
        Some("ftml-subsubsection")
      ),
      SectionLevel::Paragraph => (None,Some("ftml-paragraph")),
      SectionLevel::Subparagraph => (None,Some("ftml-subparagraph")),
    })
  }

  pub fn set_section(&mut self,lvl:SectionLevel) {
    self.init_paras();
    self.sections.inc_at(lvl);
    //leptos::logging::log!("Setting section to {} => {}",lvl,self.sections.get_untracked());
    self.resets.with_untracked(|rs| {
      //leptos::logging::log!("Resetting at {lvl}: {rs:?}");
      for (l,r) in &rs.0 {
        if *l >= lvl {
          for n in r.iter() {
            self.counters.with_untracked(|c| {
              if let Some(c) = c.get(n) {
                //leptos::logging::log!("Resetting {n}");
                c.reset();
              }
            });
          }
        }
      }
    });
    self.current = LogicalLevel::Section(lvl);
  }

  fn get_counter(all:&VecMap<(ParagraphKind,Option<Name>),Option<Name>>,kind:ParagraphKind,styles:&[Name]) -> Option<Name> {
    styles.iter().rev().find_map(|s| all.0.iter().find_map(|((k,n),v)|
      if *k == kind && n.as_ref().is_some_and(|n| *n == *s) { Some(v.as_ref()) } else { None }
    )).unwrap_or_else(|| 
      all.get(&(kind,None)).map(|o| o.as_ref()).flatten()
    ).cloned()
  }

  pub fn get_para(&mut self,kind:ParagraphKind,styles:&[Name]) -> Memo<String> {
    self.init_paras();
    self.current = LogicalLevel::Paragraph;
    let cnt = self.for_paras.with_untracked(|all_styles| {
      Self::get_counter(all_styles,kind,styles)
    });
    if let Some(cntname) = cnt {
      let Some(cnt) = self.counters.with_untracked(|cntrs| cntrs.get(&cntname).copied()) else {
        unreachable!()
      };
      cnt.inc_memo(move |i| format!("counter-set:ftml-{cntname} {i};"))
    } else {
      Memo::new(|_| String::new())
    }
  }

  pub fn get_exercise(&mut self,styles:&[Name]) -> Memo<String> {
    self.init_paras();
    self.current = LogicalLevel::Paragraph;
    Memo::new(|_| String::new())
  }

  pub fn get_slide() -> Memo<u32> {
    let counters : Self = expect_context();
    counters.init_paras();
    counters.slides.memo(|i| i)
  }
  pub fn slide_inc() -> Self {
    let mut counters : Self = expect_context();
    counters.init_paras();
    counters.slides.inc();
    counters.current = LogicalLevel::BeamerSlide;
    counters
  }

  pub fn inputref(uri:DocumentURI,id:String) -> Self {
    let mut counters : Self = expect_context();
    counters.init_paras();

    //leptos::logging::log!("Here: {uri}@{id}");

    let old_slides = counters.slides;//.0.get_untracked();
    counters.slides = counters.slides.split();
    let old_slides = old_slides.0.get_untracked().cutoff.unwrap_or_else(|| unreachable!()).set;

    let old_sections = counters.sections;
    counters.sections = counters.sections.split();
    let old_sections = old_sections.0.get_untracked().cutoff.unwrap_or_else(|| unreachable!()).set;

    let mut new_paras = VecMap::default();

    let old_paras = counters.counters.with_untracked(|v|
      v.0.iter().map(|(n,e)| {
        //leptos::logging::log!("Cloning {n}");
        let mut r = *e;
        let since = r.0.update_untracked(|e| {let r = e.since; e.since = 0;r});
        new_paras.insert(n.clone(),e.split());
        (n.clone(),(r.0.get_untracked().cutoff.unwrap_or_else(|| unreachable!()).set,since))
      }).collect::<VecMap<_,_>>()
    );
    counters.counters = RwSignal::new(new_paras);

    let ctw = expect_context::<RwSignal::<Option<Vec<TOCElem>>>>();
    let uricl = uri.clone();let idcl = id.clone();
    let children = Memo::new(move |_| {
      let uri = &uricl;
      let id = &idcl;
      ctw.with(|v| if let Some(v) = v.as_ref() {
        for e in v.iter_elems() {
          if let TOCElem::Inputref{uri:u,id:i,children:chs,..} = e {
            if u == uri && i == id {
              return Some(chs.clone());
            }
          }
        }
        None
      } else {None})
    });

    let current = counters.current;
    let max = counters.max;
    let para_map = counters.for_paras;

    Effect::new(move || 
      children.with(|ch|
        if let Some(ch) = ch.as_ref(){ 
          //leptos::logging::log!("Updating {uri}@{id}");
          para_map.with_untracked(|m|
            update(ch,current,max,&old_slides,&old_sections,&old_paras,m)
          )
        }
      )
    );

    counters
  }
}

fn update(ch:&[TOCElem],
  mut current:LogicalLevel,
  max:SectionLevel,
  old_slides:&RwSignal<u32>,
  old_sections:&RwSignal<AllSections>,
  old_paras:&VecMap<Name,(RwSignal<u16>,u16)>,
  para_map:&VecMap<(ParagraphKind,Option<Name>),Option<Name>>
) {
  let mut curr = ch.iter();
  let mut stack = SmallVec::<_,4>::new();
  
  let mut n_slides = 0;
  let mut n_sections = AllSections::default();
  let mut n_counters = old_paras.0.iter().map(|(n,(_,i))| (n.clone(),*i)).collect::<VecMap<_,_>>();

  //tracing::warn!("Updating inputref: {ch:?} in level {current:?}");

  loop {
    if let Some(c) = curr.next() {
      match c {
        TOCElem::Slide => n_slides += 1,
        TOCElem::Section{children,..} => {
          let lvl = if let LogicalLevel::Section(s) = current {
            s.inc()
          } else if current == LogicalLevel::None { max } else {
            continue
          };
          n_sections = n_sections + lvl;
          let old = std::mem::replace(&mut current,LogicalLevel::Section(lvl));
          stack.push((std::mem::replace(&mut curr,children.iter()),old)); 
        }
        TOCElem::Inputref { children,.. } => {
          //let old = std::mem::replace(&mut current,LogicalLevel::Paragraph);
          stack.push((std::mem::replace(&mut curr,children.iter()),current));
        }
        TOCElem::Paragraph { styles, kind } => {
          if let Some(n) = SectionCounters::get_counter(para_map,*kind,styles) {
            //leptos::logging::log!("Increasing counter {n}");
            *n_counters.get_or_insert_mut(n, || 0) += 1;
          }
        }

      }
    } else if let Some((next,lvl)) = stack.pop() {
      curr = next;
      current = lvl;
    } else { break }
  }

  //tracing::warn!("Seting inpuref sections to {n_sections:?}");
  //leptos::logging::log!("Setting cutoffs");
  old_slides.set(n_slides);
  old_sections.set(n_sections);
  for (n,v) in n_counters {
    //leptos::logging::log!("Patching counter {n} as {v}");
    if let Some((s,_)) = old_paras.get(&n) { s.set(v); }
  }
  
}