#[test]
fn test_link_to_other_struct() {
  use linked_data_schema::{
    LinkedDataSchema, print_linked_data_schema_for,
    reexports::{iri_s::iri, prefixmap::PrefixMap, shacl::ast::ASTSchema},
  };

  #[derive(LinkedDataSchema, Debug, PartialEq)]
  #[ld(prefix("ex" = "http://example.com/"))]
  #[ld(type = "ex:StructA")]
  struct StructA {
    #[ld("ex:field_a_0")]
    field_a_0: StructB,
  }

  #[derive(LinkedDataSchema, Debug, PartialEq)]
  #[ld(prefix("ex" = "http://example.com/"))]
  #[ld(type = "ex:StructB")]
  struct StructB {
    #[ld("ex:field_b_0")]
    field_b_0: String,
  }

  let schema: ASTSchema = StructA::shacl();

  let expected_prefix_map = {
    let mut prefix_map = PrefixMap::new();
    prefix_map
      .add_prefix("ex", iri!("http://example.com/"))
      .unwrap();
    prefix_map
  };

  assert_eq!(schema.prefixmap(), &expected_prefix_map);

  print_linked_data_schema_for!(StructA);
  print_linked_data_schema_for!(StructB);

  // let expected_shapes = HashMap::from([]);

  // assert_eq!(schema.iter().map(|(node, shape)| (node.clone(), shape.clone())).collect::<HashMap<RDFNode, Shape>>(), expected_shapes);
}
