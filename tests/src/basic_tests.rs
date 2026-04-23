#[test]
fn test_basic_usage() {
  use linked_data_schema::{
    LinkedDataSchema, print_linked_data_schema_for,
    reexports::{iri_s::iri, prefixmap::PrefixMap, shacl::ast::ASTSchema, uuid},
  };

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

  let schema: ASTSchema = Struct::shacl();

  let expected_prefix_map = {
    let mut prefix_map = PrefixMap::new();
    prefix_map
      .add_prefix("ex", iri!("http://example.com/"))
      .unwrap();
    prefix_map
  };

  assert_eq!(schema.prefixmap(), &expected_prefix_map);

  // print_linked_data_schema_for!(Struct);
  print_linked_data_schema_for!(SubStruct);

  // assert!(false)
  // let expected_shapes = HashMap::from([]);

  // assert_eq!(schema.iter().map(|(node, shape)| (node.clone(), shape.clone())).collect::<HashMap<RDFNode, Shape>>(), expected_shapes);
}
