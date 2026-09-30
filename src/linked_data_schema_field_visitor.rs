mod uuid;

use prefixmap::IriRef;
use rudof_rdf::rdf_core::term::Object;
use shacl::ast::{ASTComponent, ASTShape};
use std::collections::{HashMap, HashSet};

pub trait LinkedDataSchemaFieldVisitor {
  /// Constraints on each value of a field of this type, e.g. its datatype.
  fn value_components() -> Vec<ASTComponent>;

  /// Constraints on a field of this type: by default, exactly one value.
  fn field_components() -> Vec<ASTComponent> {
    [
      vec![ASTComponent::MinCount(1), ASTComponent::MaxCount(1)],
      Self::value_components(),
    ]
    .concat()
  }

  /// Adds the shapes referenced by the constraints, e.g. the node shape of a nested struct.
  fn add_shapes(_shapes: &mut HashMap<Object, ASTShape>) {}
}

macro_rules! field_visitor_impl {
  ($for_type:ty, $uri_datatype:literal) => {
    impl LinkedDataSchemaFieldVisitor for $for_type {
      fn value_components() -> Vec<ASTComponent> {
        use std::str::FromStr;

        vec![ASTComponent::Datatype(
          IriRef::from_str($uri_datatype).unwrap(),
        )]
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

/// An optional field keeps the constraints of its inner type, except the minimum count.
impl<S: LinkedDataSchemaFieldVisitor> LinkedDataSchemaFieldVisitor for Option<S> {
  fn value_components() -> Vec<ASTComponent> {
    S::value_components()
  }

  fn field_components() -> Vec<ASTComponent> {
    S::field_components()
      .into_iter()
      .filter(|component| !matches!(component, ASTComponent::MinCount(_)))
      .collect()
  }

  fn add_shapes(shapes: &mut HashMap<Object, ASTShape>) {
    S::add_shapes(shapes)
  }
}

/// A multivalued field only constrains its values, not their count.
impl<S: LinkedDataSchemaFieldVisitor> LinkedDataSchemaFieldVisitor for Vec<S> {
  fn value_components() -> Vec<ASTComponent> {
    S::value_components()
  }

  fn field_components() -> Vec<ASTComponent> {
    S::value_components()
  }

  fn add_shapes(shapes: &mut HashMap<Object, ASTShape>) {
    S::add_shapes(shapes)
  }
}

/// A multivalued field only constrains its values, not their count.
impl<S: LinkedDataSchemaFieldVisitor> LinkedDataSchemaFieldVisitor for HashSet<S> {
  fn value_components() -> Vec<ASTComponent> {
    S::value_components()
  }

  fn field_components() -> Vec<ASTComponent> {
    S::value_components()
  }

  fn add_shapes(shapes: &mut HashMap<Object, ASTShape>) {
    S::add_shapes(shapes)
  }
}
