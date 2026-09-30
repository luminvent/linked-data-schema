use linked_data_schema::{
  LinkedDataSchema, print_linked_data_schema_for,
  reexports::{
    iri_s::{IriS, iri},
    prefixmap::{IriRef, PrefixMap},
    rudof_rdf::rdf_core::term::Object,
    shacl::{
      ast::{ASTComponent, ASTSchema, ASTShape},
      ir::IRSchema,
    },
    uuid,
  },
};
use std::str::FromStr;

#[derive(LinkedDataSchema, Debug, PartialEq)]
#[ld(prefix("ex" = "http://example.com/"))]
#[ld(type = "ex:Struct")]
struct Struct {
  #[ld("ex:field_0")]
  field_0: String,

  #[ld("ex:field_1")]
  field_1: Option<String>,

  #[ld("ex:field_2")]
  field_2: Vec<String>,

  #[ld("ex:field_3")]
  field_3: Vec<u8>,

  #[ld("ex:field_4")]
  field_4: u64,

  #[ld("ex:field_5")]
  field_5: uuid::Uuid,

  #[ld("ex:field_6")]
  field_6: SubStruct,

  #[ld("ex:field_7")]
  field_7: Option<SubStruct>,

  #[ld("ex:field_8")]
  field_8: Vec<SubStruct>,
}

#[derive(LinkedDataSchema, Debug, PartialEq)]
#[ld(prefix("ex" = "http://example.com/"))]
#[ld(type = "ex:SubStruct")]
struct SubStruct {
  #[ld("ex:sub_field_0")]
  sub_field_0: String,
}

fn object(iri: &str) -> Object {
  Object::Iri(IriS::from_str(iri).unwrap())
}

fn datatype(iri: &str) -> ASTComponent {
  ASTComponent::Datatype(IriRef::from_str(iri).unwrap())
}

fn property_components(schema: &ASTSchema, field: &str) -> Vec<ASTComponent> {
  match schema.get_shape(&object(&format!(
    "http://example.com/StructShape/ex:{field}"
  ))) {
    Some(ASTShape::PropertyShape(shape)) => shape.components().clone(),
    shape => panic!("expected a property shape for {field}, got {shape:?}"),
  }
}

#[test]
fn test_basic_usage() {
  let schema: ASTSchema = Struct::shacl();

  let expected_prefix_map = {
    let mut prefix_map = PrefixMap::new();
    prefix_map
      .add_prefix("ex", iri!("http://example.com/"))
      .unwrap();
    prefix_map
  };

  assert_eq!(schema.prefixmap(), &expected_prefix_map);

  print_linked_data_schema_for!(Struct);
  print_linked_data_schema_for!(SubStruct);
}

#[test]
fn test_field_components() {
  use ASTComponent::{MaxCount, MinCount, Node, Pattern};

  let schema = Struct::shacl();
  let string = || datatype("http://www.w3.org/2001/XMLSchema#string");
  let sub_struct_node = || Node(object("http://example.com/SubStructShape"));

  assert_eq!(
    property_components(&schema, "field_0"),
    [MinCount(1), MaxCount(1), string()]
  );
  assert_eq!(
    property_components(&schema, "field_1"),
    [MaxCount(1), string()]
  );
  assert_eq!(property_components(&schema, "field_2"), [string()]);
  assert_eq!(
    property_components(&schema, "field_3"),
    [datatype("http://www.w3.org/2001/XMLSchema#unsignedByte")]
  );
  assert_eq!(
    property_components(&schema, "field_4"),
    [
      MinCount(1),
      MaxCount(1),
      datatype("http://www.w3.org/2001/XMLSchema#unsignedLong")
    ]
  );
  assert_eq!(
    property_components(&schema, "field_5"),
    [
      MinCount(1),
      MaxCount(1),
      string(),
      Pattern {
        pattern:
          "^urn:uuid:[0-9a-f]{8}-[0-9a-f]{4}-[0-5][0-9a-f]{3}-[089ab][0-9a-f]{3}-[0-9a-f]{12}$"
            .to_string(),
        flags: Some("i".to_string()),
      }
    ]
  );
  assert_eq!(
    property_components(&schema, "field_6"),
    [MinCount(1), MaxCount(1), sub_struct_node()]
  );
  assert_eq!(
    property_components(&schema, "field_7"),
    [MaxCount(1), sub_struct_node()]
  );
  assert_eq!(property_components(&schema, "field_8"), [sub_struct_node()]);
}

#[test]
fn test_node_shape_is_identified_by_its_iri() {
  let schema = Struct::shacl();

  let Some(ASTShape::NodeShape(node_shape)) =
    schema.get_shape(&object("http://example.com/StructShape"))
  else {
    panic!("expected the node shape under its IRI");
  };

  assert_eq!(node_shape.id(), &object("http://example.com/StructShape"));
  assert_eq!(node_shape.property_shapes().len(), 9);

  // 1 node shape and 9 property shapes, plus the nested `SubStruct` node shape and property shape.
  assert_eq!(schema.iter().count(), 12);
  assert!(matches!(
    schema.get_shape(&object("http://example.com/SubStructShape")),
    Some(ASTShape::NodeShape(_))
  ));
}

#[test]
fn test_schema_is_deterministic() {
  let shapes = |schema: ASTSchema| {
    let mut shapes = schema
      .iter()
      .map(|(id, shape)| format!("{id:?} {shape:?}"))
      .collect::<Vec<_>>();
    shapes.sort();
    shapes
  };

  assert_eq!(shapes(Struct::shacl()), shapes(Struct::shacl()));
}

#[test]
fn test_schema_is_valid() {
  IRSchema::try_from(Struct::shacl()).unwrap();
  IRSchema::try_from(SubStruct::shacl()).unwrap();
}
