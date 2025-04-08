mod uuid;

use iri_s::iri;
use prefixmap::IriRef;
use shacl_ast::component::Component;

pub trait LinkedDataSchemaFieldVisitor {
  fn components() -> Vec<Component> {
    vec![]
  }

  fn type_iri_ref() -> Option<IriRef>;
}

macro_rules! field_visitor_impl {
  ($for_type:ty, $xsd_type:literal) => {
    impl LinkedDataSchemaFieldVisitor for $for_type {
      fn components() -> Vec<Component> {
        vec![
          Component::MinCount(1),
          Component::MaxCount(1),
          Component::Datatype(IriRef::iri(iri!($xsd_type))),
        ]
      }

      fn type_iri_ref() -> Option<IriRef> {
        Some(IriRef::iri(iri!($xsd_type)))
      }
    }
  };
}

field_visitor_impl!(String, "xsd:string");
field_visitor_impl!(u8, "xsd:unsignedByte");
field_visitor_impl!(i8, "xsd:byte");
field_visitor_impl!(u16, "xsd:unsignedShort");
field_visitor_impl!(i16, "xsd:short");
field_visitor_impl!(u32, "xsd:unsignedInt");
field_visitor_impl!(i32, "xsd:int");
field_visitor_impl!(u64, "xsd:unsignedLong");
field_visitor_impl!(i64, "xsd:long");
field_visitor_impl!(usize, "xsd:nonNegativeInteger");
field_visitor_impl!(isize, "xsd:integer");
field_visitor_impl!(f32, "xsd:float");
field_visitor_impl!(f64, "xsd:double");

impl<S: LinkedDataSchemaFieldVisitor> LinkedDataSchemaFieldVisitor for Option<S> {
  fn components() -> Vec<Component> {
    let datatype = S::type_iri_ref().unwrap();

    vec![Component::MaxCount(1), Component::Datatype(datatype)]
  }

  fn type_iri_ref() -> Option<IriRef> {
    None
  }
}

impl<S: LinkedDataSchemaFieldVisitor> LinkedDataSchemaFieldVisitor for Vec<S> {
  fn components() -> Vec<Component> {
    let datatype = S::type_iri_ref().unwrap();

    vec![Component::Datatype(datatype)]
  }

  fn type_iri_ref() -> Option<IriRef> {
    None
  }
}
