use linked_data_schema::{
  LinkedDataSchema,
  reexports::{
    iri_s::IriS,
    prefixmap::IriRef,
    rudof_rdf::rdf_core::{SHACLPath, term::Object},
    shacl::{
      ast::{ASTComponent, ASTSchema, ASTShape},
      ir::IRSchema,
      types::{NodeKind, Value},
    },
  },
};
use std::str::FromStr;

#[allow(dead_code)]
#[derive(LinkedDataSchema)]
#[ld(prefix("ex" = "http://example.com/"))]
#[ld(type = "ex:Struct")]
struct Struct {
  #[ld("ex:color")]
  color: Color,

  #[ld("ex:value")]
  value: Option<Content>,
}

#[allow(dead_code)]
#[derive(LinkedDataSchema)]
#[ld(prefix("ex" = "http://example.com/"))]
enum Color {
  #[ld("ex:Red")]
  Red,
  #[ld("ex:Blue")]
  Blue,
}

#[allow(dead_code)]
#[derive(LinkedDataSchema)]
#[ld(prefix("ex" = "http://example.com/"))]
enum Content {
  #[ld("ex:text")]
  Text(String),
  #[ld("ex:number")]
  Number(#[ld("ex:amount")] Option<i64>),
}

fn iri(iri: &str) -> IriS {
  IriS::from_str(iri).unwrap()
}

fn property_shape(schema: &ASTSchema, id: &Object) -> (SHACLPath, Vec<ASTComponent>) {
  match schema.get_shape(id) {
    Some(ASTShape::PropertyShape(shape)) => (shape.path().clone(), shape.components().clone()),
    shape => panic!("expected a property shape for {id:?}, got {shape:?}"),
  }
}

#[test]
fn test_closed_list_field() {
  use ASTComponent::{In, MaxCount, MinCount};

  let schema = Struct::shacl();
  let (_, components) = property_shape(
    &schema,
    &Object::Iri(iri("http://example.com/StructShape/ex:color")),
  );

  assert_eq!(
    components,
    [
      MinCount(1),
      MaxCount(1),
      ASTComponent::NodeKind(NodeKind::Iri),
      In(vec![
        Value::Iri(IriRef::iri(iri("http://example.com/Red"))),
        Value::Iri(IriRef::iri(iri("http://example.com/Blue"))),
      ]),
    ]
  );
}

#[test]
fn test_tagged_union_field() {
  use ASTComponent::{Datatype, MaxCount, MinCount, Node, Xone};

  let schema = Struct::shacl();
  let blank_node = |label: &str| Object::BlankNode(label.to_string());

  let (_, components) = property_shape(
    &schema,
    &Object::Iri(iri("http://example.com/StructShape/ex:value")),
  );
  assert_eq!(components, [MaxCount(1), Node(blank_node("ContentShape"))]);

  let Some(ASTShape::NodeShape(content_shape)) = schema.get_shape(&blank_node("ContentShape"))
  else {
    panic!("expected the node shape of the enum");
  };
  assert_eq!(
    content_shape.components(),
    &[Xone(vec![
      blank_node("ContentShape_0"),
      blank_node("ContentShape_1")
    ])]
  );

  assert_eq!(
    property_shape(&schema, &blank_node("ContentShape_0_value")),
    (
      SHACLPath::iri(iri("http://example.com/text")),
      vec![
        MinCount(1),
        MaxCount(1),
        Datatype(IriRef::iri(iri("http://www.w3.org/2001/XMLSchema#string"))),
      ]
    )
  );

  // `ex:amount` is on the value, `ex:number` on the variant: `_:s ex:amount [ ex:number ?o ]`.
  assert_eq!(
    property_shape(&schema, &blank_node("ContentShape_1_value")),
    (
      SHACLPath::sequence(vec![
        SHACLPath::iri(iri("http://example.com/amount")),
        SHACLPath::iri(iri("http://example.com/number")),
      ]),
      vec![
        MaxCount(1),
        Datatype(IriRef::iri(iri("http://www.w3.org/2001/XMLSchema#long"))),
      ]
    )
  );
}

#[test]
fn test_enum_schemas_are_valid() {
  IRSchema::try_from(Struct::shacl()).unwrap();
  IRSchema::try_from(Content::shacl()).unwrap();
  IRSchema::try_from(Color::shacl()).unwrap();
}
