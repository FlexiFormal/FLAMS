#![allow(clippy::wildcard_imports)]

use crate::{narration::paragraphs::ParagraphKind, uris::{DocumentURI, SymbolURI,DocumentElementURI}};

#[derive(Copy,Clone,Debug)]
#[cfg_attr(feature="serde", derive(serde::Serialize,serde::Deserialize))]
#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub enum QueryFilter {
  Symbols,
  Fragments(FragmentQueryOpts)
}
impl Default for QueryFilter {
  fn default() -> Self {
      Self::Fragments(FragmentQueryOpts::default())
  }
}

const fn get_true() -> bool {true}
#[allow(clippy::struct_excessive_bools)]
#[derive(Copy,Clone,Debug,PartialEq,Eq)]
#[cfg_attr(feature="serde", derive(serde::Serialize,serde::Deserialize))]
#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct FragmentQueryOpts {
  #[cfg_attr(feature="serde",serde(default="get_true"))]
  pub allow_documents:bool,
  #[cfg_attr(feature="serde",serde(default="get_true"))]
  pub allow_paragraphs:bool,
  #[cfg_attr(feature="serde",serde(default="get_true"))]
  pub allow_definitions:bool,
  #[cfg_attr(feature="serde",serde(default="get_true"))]
  pub allow_examples:bool,
  #[cfg_attr(feature="serde",serde(default="get_true"))]
  pub allow_assertions:bool,
  #[cfg_attr(feature="serde",serde(default="get_true"))]
  pub allow_exercises:bool,
  #[cfg_attr(feature="serde",serde(default))]
  pub definition_like_only:bool
}

impl Default for FragmentQueryOpts {
  fn default() -> Self {
      Self {
        allow_documents:true,
        allow_paragraphs:true,
        allow_definitions:true,
        allow_examples:true,
        allow_assertions:true,
        allow_exercises:true,
        definition_like_only:false
      }
  }
}


#[derive(Debug,Clone)]
#[cfg_attr(feature="serde", derive(serde::Serialize,serde::Deserialize))]
#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub enum SearchResult {
  Document(DocumentURI),
  Paragraph {
    uri:DocumentElementURI,
    fors:Vec<SymbolURI>,
    def_like:bool,
    kind:SearchResultKind
  }
}

#[derive(Copy,Clone,Debug)]
#[cfg_attr(feature="serde", derive(serde::Serialize,serde::Deserialize))]
#[cfg_attr(feature = "wasm", derive(tsify_next::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub enum SearchResultKind {
  Document = 0,
  Paragraph = 1,
  Definition = 2,
  Example = 3,
  Assertion = 4,
  Exercise = 5
}
impl SearchResultKind {
  #[must_use]
  pub const fn as_str(&self) -> &'static str {
      match self {
        Self::Document => "Document",
        Self::Paragraph => "Paragraph",
        Self::Definition => "Definition",
        Self::Example => "Example",
        Self::Assertion => "Assertion",
        Self::Exercise => "Exercise"
      }
  }
}

impl From<SearchResultKind> for u64 {
  fn from(value: SearchResultKind) -> Self {
    match value {
      SearchResultKind::Document => 0,
      SearchResultKind::Paragraph => 1,
      SearchResultKind::Definition => 2,
      SearchResultKind::Example => 3,
      SearchResultKind::Assertion => 4,
      SearchResultKind::Exercise => 5
    }
  }
}

impl TryFrom<u64> for SearchResultKind {
  type Error = ();
  fn try_from(value: u64) -> Result<Self, Self::Error> {
    Ok(match value {
      0 => Self::Document,
      1 => Self::Paragraph,
      2 => Self::Definition,
      3 => Self::Example,
      4 => Self::Assertion,
      5 => Self::Exercise,
      _ => return Err(())
    })
  }
}
impl TryFrom<ParagraphKind> for SearchResultKind {
  type Error = ();
  fn try_from(value: ParagraphKind) -> Result<Self, Self::Error> {
      Ok(match value {
        ParagraphKind::Assertion => Self::Assertion,
        ParagraphKind::Definition => Self::Definition,
        ParagraphKind::Example => Self::Example,
        ParagraphKind::Paragraph => Self::Paragraph,
        _ => return Err(())
      })
  }
}

#[cfg_attr(feature="serde", derive(serde::Serialize,serde::Deserialize))]
#[derive(Debug,Clone)]
pub enum SearchIndex {
  Document {
    uri:DocumentURI,
    title:Option<String>,
    body:String
  },
  Paragraph {
    uri:DocumentElementURI,
    kind:SearchResultKind,
    definition_like:bool,
    title:Option<String>,
    fors:Vec<SymbolURI>,
    body:String
  }
}


#[cfg(feature="tantivy")]
mod tantivy_i {
  use crate::{narration::{documents::{Document, UncheckedDocument}, paragraphs::LogicalParagraph, DocumentElement}, CheckingState};
  use super::*;

  pub struct SearchSchema {
    #[allow(dead_code)]
    pub schema:tantivy::schema::Schema,
    uri: tantivy::schema::Field,
    kind: tantivy::schema::Field,
    title: tantivy::schema::Field,
    body: tantivy::schema::Field,
    fors: tantivy::schema::Field,
    def_like: tantivy::schema::Field
  }
  impl SearchSchema {
    #[inline]#[must_use]
    pub fn get() -> &'static Self { &SCHEMA }
  }
  
  lazy_static::lazy_static! {
    static ref SCHEMA : SearchSchema = {
      use tantivy::schema::{Schema,INDEXED,STORED,TEXT};
/*
      let text_field_indexing = tantivy::schema::TextFieldIndexing::default()
        .set_tokenizer("ngram3")
        .set_index_option(tantivy::schema::IndexRecordOption::WithFreqsAndPositions);
      let txt_opts = tantivy::schema::TextOptions::default().set_indexing_options(text_field_indexing);
       */
    
      let mut schema = Schema::builder();
      let kind = schema.add_u64_field("kind", INDEXED|STORED);
      let uri = schema.add_text_field("uri", STORED);
      let def_like = schema.add_bool_field("deflike", INDEXED|STORED);
      let fors = schema.add_text_field("for", STORED);
      let title = schema.add_text_field("title", TEXT);
      let body = schema.add_text_field("body", TEXT);//txt_opts);//TEXT);
    
      let schema = schema.build();
      SearchSchema {
        schema,uri,kind,title,body,fors,def_like
      }
    };
  }

  impl QueryFilter {
    #[must_use]
    pub fn to_query(self,query:&str,index:&tantivy::Index) -> Option<Box<dyn tantivy::query::Query>> {
      match self {
        Self::Fragments(f) => f.to_query(query,index),
        Self::Symbols => FragmentQueryOpts{ allow_documents:false, allow_paragraphs:true, allow_definitions:true, allow_examples:false, allow_assertions:true, allow_exercises:false, definition_like_only:true}
          .to_query(query,index)
      }
    }
  }

  impl FragmentQueryOpts {
    fn to_query(self,query:&str,index:&tantivy::Index) -> Option<Box<dyn tantivy::query::Query>> {
      use std::fmt::Write;
      let Self {allow_documents,allow_paragraphs,allow_definitions,allow_examples,allow_assertions,allow_exercises,definition_like_only} = self;
      let mut s = String::new();
      if !allow_documents || !allow_paragraphs || !allow_definitions || !allow_examples || !allow_assertions || !allow_exercises {
        s.push('(');
        let mut had_first = false;
        if allow_documents {
          had_first = true;
          s.push_str("kind:0");
        }
        if allow_paragraphs {
          s.push_str(if had_first {" OR kind:1"} else {"kind:1"});
          had_first = true;
        }
        if allow_definitions {
          s.push_str(if had_first {" OR kind:2"} else {"kind:2"});
          had_first = true;
        }
        if allow_examples {
          s.push_str(if had_first {" OR kind:3"} else {"kind:3"});
          had_first = true;
        }
        if allow_assertions {
          s.push_str(if had_first {" OR kind:4"} else {"kind:4"});
          had_first = true;
        }
        if allow_exercises {
          s.push_str(if had_first {" OR kind:5"} else {"kind:5"});
        }
        s.push_str(") AND ");
      }
      if definition_like_only {
        s.push_str("deflike:true AND ");
      }
      write!(s,"({query})").ok()?;
      let mut parser = tantivy::query::QueryParser::for_index(index,vec![SCHEMA.title,SCHEMA.body]);
      parser.set_field_fuzzy(SCHEMA.body,false,1,true);
      parser.set_conjunction_by_default();
      parser.parse_query(&s).ok()
    }
  }

  impl tantivy::schema::document::ValueDeserialize for SearchResultKind {
    fn deserialize<'de, D>(deserializer: D) -> Result<Self, tantivy::schema::document::DeserializeError>
        where D: tantivy::schema::document::ValueDeserializer<'de> {
        deserializer.deserialize_u64()?.try_into().map_err(|()| tantivy::schema::document::DeserializeError::custom(""))
    }
  }

  impl tantivy::schema::document::DocumentDeserialize for SearchResult {
    fn deserialize<'de, D>(mut deserializer: D) -> Result<Self, tantivy::schema::document::DeserializeError>
        where D: tantivy::schema::document::DocumentDeserializer<'de> {
          macro_rules! next {
            () => {{
              let Some((_,r)) = deserializer.next_field()? else {
                return Err(tantivy::schema::document::DeserializeError::custom("Missing value"))
              };
              r
            }};
            (!) => {{
              let Some((_,Wrapper(r))) = deserializer.next_field()? else {
                return Err(tantivy::schema::document::DeserializeError::custom("Missing value"))
              };
              r
            }}
          }
          let kind =next!();
          match kind {
            SearchResultKind::Document => Ok(Self::Document(next!())),
            kind => {
              let uri = next!();
              let def_like = next!(!);
              let mut fors = Vec::new();
              while let Some((_,s)) = deserializer.next_field()? {
                fors.push(s);
              };
              Ok(Self::Paragraph{ uri,def_like,kind,fors })
            }
          }
    }
  }

  #[derive(Debug)]
  struct Wrapper<T>(T);
  impl tantivy::schema::document::ValueDeserialize for Wrapper<bool> {
    fn deserialize<'de, D>(deserializer: D) -> Result<Self, tantivy::schema::document::DeserializeError>
        where D: tantivy::schema::document::ValueDeserializer<'de> {
        Ok(Self(deserializer.deserialize_bool()?))
    }
  }

  impl SearchIndex {
    #[must_use]
    pub fn html_to_search_text(html:&str) -> Option<String> {
      html2text::from_read(html.as_bytes(),usize::MAX / 3).ok()
    }
  }

  impl From<SearchIndex> for tantivy::TantivyDocument {
    fn from(value: SearchIndex) -> Self {
      let mut ret = Self::default();
      match value {
        SearchIndex::Document { uri, title, body } => {
          ret.add_u64(SCHEMA.kind, SearchResultKind::Document.into());
          ret.add_text(SCHEMA.uri,uri.to_string());
          if let Some(t) = title {
            ret.add_text(SCHEMA.title,t);
          }
          ret.add_text(SCHEMA.body, body);
        }
        SearchIndex::Paragraph { uri, kind, definition_like, title, fors, body } => {
          ret.add_u64(SCHEMA.kind, kind.into());
          ret.add_text(SCHEMA.uri,uri.to_string());
          ret.add_bool(SCHEMA.def_like, definition_like);      
          for f in fors {
            //write!(trace,"\n   FOR: {}",f);
            ret.add_text(SCHEMA.fors,f.to_string());
          }
          if let Some(t) = title {
            ret.add_text(SCHEMA.title,t);
          }
          ret.add_text(SCHEMA.body, body);

        }
      }
      ret
    }
  }

  impl Document {
    pub fn search_index(&self,html:&str) -> Option<SearchIndex> {
      let title = self.title().and_then(|s|
        SearchIndex::html_to_search_text(s).or_else(|| {
            tracing::error!("Failed to plain textify title: {s}");
            None
        })
      );
      let Some(body) = SearchIndex::html_to_search_text(html) else {
        tracing::error!("Failed to plain textify body of {}",self.uri());
        return None
      };
      Some(SearchIndex::Document { uri: self.uri().clone(), title, body })
    }

    #[must_use]
    pub fn all_searches(&self,html:&str) -> Vec<SearchIndex> {
      let mut ret = vec![];
      if let Some(s) = self.search_index(html) {
        ret.push(s);
      }
      for e in self.dfs() {
        if let DocumentElement::Paragraph(p) = e {
          if let Some(s) = p.search_index(html)  {
              ret.push(s);
          }
        }
      }
      ret
    }
  }


  impl UncheckedDocument {
    pub fn search_index(&self,html:&str) -> Option<SearchIndex> {
      let title = self.title.as_ref().and_then(|s|
        SearchIndex::html_to_search_text(s).or_else(|| {
            tracing::error!("Failed to plain textify title: {s}");
            None
        })
      );
      let Some(body) = SearchIndex::html_to_search_text(html) else {
        tracing::error!("Failed to plain textify body of {}",self.uri);
        return None
      };
      Some(SearchIndex::Document { uri: self.uri.clone(), title, body })
    }

    #[must_use]
    pub fn all_searches(&self,html:&str) -> Vec<SearchIndex> {
      let mut ret = vec![];
      if let Some(s) = self.search_index(html) {
        ret.push(s);
      }
      for e in self.dfs() {
        if let DocumentElement::Paragraph(p) = e {
          if let Some(s) = p.search_index(html)  {
              ret.push(s);
          }
        }
      }
      ret
    }

  }

  impl<S:CheckingState> LogicalParagraph<S> {
    pub fn search_index(&self,html:&str) -> Option<SearchIndex> {
      let title = self.title.and_then(|range|
        html.get(range.start..range.end).map_or_else(
          || {
            tracing::error!("Failed to plain textify title: Range {range:?}");
            None
          },
          |s| SearchIndex::html_to_search_text(s).or_else(|| {
            tracing::error!("Failed to plain textify title: {s}");
            None
        })
        )
      );
      let Some(body) = html.get(self.range.start..self.range.end) else {
        tracing::error!("Failed to plain textify body of {}",self.uri);
        return None
      };
      let Some(body) = SearchIndex::html_to_search_text(body) else {
        tracing::error!("Failed to plain textify body of {}",self.uri);
        return None
      };
      let fors = self.fors.iter().map(|(f,_)| f.clone()).collect();

      let Ok(kind) = self.kind.try_into() else {return None}; 
      let definition_like = self.kind.is_definition_like(&self.styles);
      
      Some(SearchIndex::Paragraph { uri:self.uri.clone(), kind, definition_like, title, fors, body })
    }
  }

}
#[cfg(feature="tantivy")]
pub use tantivy_i::*;