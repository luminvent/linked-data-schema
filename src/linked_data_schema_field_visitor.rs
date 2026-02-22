mod uuid;

use prefixmap::IriRef;
use shacl_ast::component::Component;
use std::collections::HashSet;

pub trait LinkedDataSchemaFieldVisitor {
  fn field_components() -> Vec<Component> {
    vec![]
  }

  fn type_iri_ref() -> Option<IriRef>;
}

macro_rules! field_visitor_impl {
  ($for_type:ty, $uri_datatype:literal) => {
    impl LinkedDataSchemaFieldVisitor for $for_type {
      fn field_components() -> Vec<Component> {
        use std::str::FromStr;

        vec![
          Component::MinCount(1),
          Component::MaxCount(1),
          Component::Datatype(IriRef::from_str($uri_datatype).unwrap()),
        ]
      }

      fn type_iri_ref() -> Option<IriRef> {
        use std::str::FromStr;

        IriRef::from_str($uri_datatype).ok()
      }
    }
  };
}

field_visitor_impl!(String, "http://www.w3.org/2001/XMLSchema#string");
field_visitor_impl!(bool, "http://www.w3.org/2001/XMLSchema#boolean");
field_visitor_impl!(u8, "http://www.w3.org/2001/XMLSchema#unsignedByte");
field_visitor_impl!(i8, "http://www.w3.org/2001/XMLSchema#byte");
field_visitor_impl!(u16, "http://www.w3.org/2001/XMLSchema#unsignedShort");
field_visitor_impl!(i16, "http://www.w3.org/2001/XMLSchema#short");
field_visitor_impl!(u32, "http://www.w3.org/2001/XMLSchema#unsignedInt");
field_visitor_impl!(i32, "http://www.w3.org/2001/XMLSchema#int");
field_visitor_impl!(u64, "http://www.w3.org/2001/XMLSchema#unsignedLong");
field_visitor_impl!(i64, "http://www.w3.org/2001/XMLSchema#long");
field_visitor_impl!(usize, "http://www.w3.org/2001/XMLSchema#nonNegativeInteger");
field_visitor_impl!(isize, "http://www.w3.org/2001/XMLSchema#integer");
field_visitor_impl!(f32, "http://www.w3.org/2001/XMLSchema#float");
field_visitor_impl!(f64, "http://www.w3.org/2001/XMLSchema#double");

impl<S: LinkedDataSchemaFieldVisitor> LinkedDataSchemaFieldVisitor for Option<S> {
  fn field_components() -> Vec<Component> {
    if let Some(datatype) = S::type_iri_ref() {
      [
        S::field_components(),
        vec![Component::MaxCount(1), Component::Datatype(datatype)],
      ]
      .concat()
    } else {
      vec![]
    }
  }

  fn type_iri_ref() -> Option<IriRef> {
    None
  }
}

impl<S: LinkedDataSchemaFieldVisitor> LinkedDataSchemaFieldVisitor for Vec<S> {
  fn field_components() -> Vec<Component> {
    if let Some(datatype) = S::type_iri_ref() {
      [S::field_components(), vec![Component::Datatype(datatype)]].concat()
    } else {
      vec![]
    }
  }

  fn type_iri_ref() -> Option<IriRef> {
    None
  }
}

impl<S: LinkedDataSchemaFieldVisitor> LinkedDataSchemaFieldVisitor for HashSet<S> {
  fn field_components() -> Vec<Component> {
    if let Some(datatype) = S::type_iri_ref() {
      [S::field_components(), vec![Component::Datatype(datatype)]].concat()
    } else {
      vec![]
    }
  }

  fn type_iri_ref() -> Option<IriRef> {
    None
  }
}
