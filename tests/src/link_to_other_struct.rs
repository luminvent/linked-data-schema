#[test]
fn test_link_to_other_struct() {
  use linked_data_schema::{
    LinkedDataSchema, print_linked_data_schema_for,
    reexports::{
      prefixmap::PrefixMap,
      srdf::{RDFFormat, SRDFGraph},
    },
  };
  use shacl_rdf::ShaclWriter;
  use std::collections::HashMap;
  use std::io::Cursor;

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

  let schema = StructA::shacl();

  let expected_prefix_map =
    PrefixMap::from_hashmap(&HashMap::from([("ex", "http://example.com/")])).unwrap();

  assert_eq!(schema.prefix_map(), expected_prefix_map);

  print_linked_data_schema_for!(StructA);
  print_linked_data_schema_for!(StructB);

  // let expected_shapes = HashMap::from([]);

  // assert_eq!(schema.iter().map(|(node, shape)| (node.clone(), shape.clone())).collect::<HashMap<RDFNode, Shape>>(), expected_shapes);
}
