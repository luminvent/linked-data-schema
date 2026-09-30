use linked_data_schema::{
  LinkedDataSchema,
  reexports::{
    iri_s::{IriS, iri},
    prefixmap::{IriRef, PrefixMap},
    rudof_rdf::rdf_core::term::Object,
    shacl::{
      ast::{ASTComponent, ASTSchema, ASTShape},
      ir::IRSchema,
    },
  },
};
use std::str::FromStr;

#[derive(LinkedDataSchema, Debug, PartialEq)]
#[ld(prefix("ex" = "http://example.com/"))]
#[ld(type = "ex:StructA")]
struct StructA {
  #[ld("ex:name")]
  name: String,

  #[ld("ex:field_a_0")]
  field_a_0: StructB,
}

#[derive(LinkedDataSchema, Debug, PartialEq)]
#[ld(prefix("ex" = "http://example.com/"))]
#[ld(type = "ex:StructB")]
struct StructB {
  #[ld("ex:name")]
  name: Option<String>,

  #[ld("ex:field_b_0")]
  field_b_0: Vec<StructA>,
}

fn object(iri: &str) -> Object {
  Object::Iri(IriS::from_str(iri).unwrap())
}

fn property_components(schema: &ASTSchema, property_shape: &str) -> Vec<ASTComponent> {
  match schema.get_shape(&object(property_shape)) {
    Some(ASTShape::PropertyShape(shape)) => shape.components().clone(),
    shape => panic!("expected a property shape for {property_shape}, got {shape:?}"),
  }
}

#[test]
fn test_link_to_other_struct() {
  let schema: ASTSchema = StructA::shacl();

  let expected_prefix_map = {
    let mut prefix_map = PrefixMap::new();
    prefix_map
      .add_prefix("ex", iri!("http://example.com/"))
      .unwrap();
    prefix_map
  };

  assert_eq!(schema.prefixmap(), &expected_prefix_map);

  // Both node shapes and their 2 property shapes each, although the types reference each other.
  assert_eq!(schema.iter().count(), 6);
  IRSchema::try_from(schema).unwrap();
}

#[test]
fn test_shared_predicate_keeps_constraints_per_struct() {
  use ASTComponent::{Datatype, MaxCount, MinCount};

  let schema = StructA::shacl();
  let string = || Datatype(IriRef::from_str("http://www.w3.org/2001/XMLSchema#string").unwrap());

  assert_eq!(
    property_components(&schema, "http://example.com/StructAShape/ex:name"),
    [MinCount(1), MaxCount(1), string()]
  );
  assert_eq!(
    property_components(&schema, "http://example.com/StructBShape/ex:name"),
    [MaxCount(1), string()]
  );
}
