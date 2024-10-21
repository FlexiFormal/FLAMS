pub mod reexport {
    pub use inventory::{collect, iter, submit};
    pub use paste::paste;
}

#[macro_export]
macro_rules! global {
  (NEW $({$path:path})? $structname:ty; $instance:ident [$($arg:expr),*]) => {
    $crate::globals::reexport::paste!{
      pub static [<$instance:snake:upper>]: $($path::)?[<$structname Id>] = $($path::)?[<$structname Id>](&$($path::)?$structname::__new__(
        stringify!($instance) $(,$arg)*
      ));
      $crate::globals::reexport::submit!([<$instance:snake:upper>]);
    }
  };
  (SER?
    $(#[$meta:meta])*
    $structname:ident {
      $mainname:ident
      $(, $name:ident : $type:ty )*
    }
  ) => {
    #[cfg(feature="serde")]
    $crate::global!{SER
      $(#[$meta])*
      $structname { $mainname $(,$name : $type)*}
    }
    #[cfg(not(feature="serde"))]
    $crate::global!{@BASE
      $(#[$meta])*
      $structname { $mainname $(,$name : $type)*}
    }
  };
  (SER
    $(#[$meta:meta])*
    $structname:ident {
      $mainname:ident
      $(, $name:ident : $type:ty )*
    }
  ) => {
    $crate::global!{@BASE
      $(#[$meta])*
      $structname { $mainname $(,$name : $type)*}
    }

    $crate::globals::reexport::paste!{
      impl ::serde::Serialize for [<$structname Id>] {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: ::serde::Serializer {
          serializer.serialize_str(self.0.$mainname)
        }
      }

      impl<'de> ::serde::Deserialize<'de> for [<$structname Id>] {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: ::serde::Deserializer<'de> {
            let s = String::deserialize(deserializer)?;
            $structname::get_from_str(&s)
                .map_or_else(
                  || Err(::serde::de::Error::custom("Unknown source file type")),
                  Ok
                )
        }
      }
    }
  };
  (
    $(#[$meta:meta])*
    $structname:ident {
      $mainname:ident
      $(, $name:ident : $type:ty )*
    }
  ) => {
    $crate::global!{@BASE
      $(#[$meta])*
      $structname { $mainname $(,$name : $type)*}
    }
  };
  (@BASE
    $(#[$meta:meta])*
    $structname:ident {
      $mainname:ident
      $(, $name:ident : $type:ty )*
    }
  ) => {$crate::globals::reexport::paste!{

    #[derive(Copy,Clone,Debug)]
    pub struct [< $structname Id >](pub &'static $structname);
    impl std::fmt::Display for [< $structname Id >] {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0.$mainname)
      }
    }
    impl Eq for [< $structname Id >] {}
    impl PartialEq for [< $structname Id >] {
      #[inline]
      fn eq(&self, other: &Self) -> bool {
          self.0.$mainname.as_ptr() == other.0.$mainname.as_ptr()
      }
    }
    impl std::hash::Hash for [< $structname Id >] {
      #[inline]
      fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
          self.0.$mainname.as_ptr().hash(state);
      }
    }
    impl std::convert::AsRef<$structname> for [< $structname Id >] {
      #[inline]
      fn as_ref(&self) -> &'static $structname { self.0 }
    }
    impl std::ops::Deref for [< $structname Id >] {
      type Target = $structname;
      #[inline]
      fn deref(&self) -> &'static $structname { self.0 }
    }


    $(#[$meta])*
    #[derive(Debug)]
    pub struct $structname {
      $mainname:&'static str
      $( ,$name:$type )*
    }
    $crate::globals::reexport::collect!([< $structname Id >]);

    impl $structname {
      #[inline]#[must_use] pub const fn __new__($mainname:&'static str $(,$name:$type)*) -> Self {
        Self { $mainname $(,$name)* }
      }
      #[inline] pub fn all() -> impl Iterator<Item=&'static Self> {
        $crate::globals::reexport::iter::<[< $structname Id >]>.into_iter().map(|e| e.0)
      }

      #[inline]#[must_use] pub fn get_from_str(s:&str) -> Option<[< $structname Id >]> {
        Self::all().find(|i| i.$mainname.eq_ignore_ascii_case(s)).map([< $structname Id >])
      }

      #[inline]#[must_use] pub const fn $mainname(&self) -> &'static str { self.$mainname }
      $(
        #[inline]#[must_use] pub const fn $name(&self) -> &$type { &self.$name }
      )*
    }

    impl std::fmt::Display for $structname {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.$mainname)
      }
    }

    impl std::hash::Hash for $structname {
      #[inline]
      fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
          self.$mainname.hash(state);
      }
    }
    impl PartialEq for $structname {
      #[inline]
      fn eq(&self, other: &Self) -> bool {
          self.$mainname.as_ptr() == other.$mainname.as_ptr()
      }
    }
  }};
}
