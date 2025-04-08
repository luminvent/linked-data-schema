mod linked_data_schema_field_visitor;

pub use linked_data_schema_derive::LinkedDataSchema;
pub use linked_data_schema_field_visitor::LinkedDataSchemaFieldVisitor;
use shacl_ast::Schema;
use shacl_ast::component::Component;
use srdf::SRDFGraph;

pub mod reexports {
  pub use iri_s;
  pub use prefixmap;
  pub use shacl_ast;
  pub use srdf;
  pub use uuid;
}

pub trait LinkedDataSchema {
  fn shacl() -> Schema<SRDFGraph>;

  fn components() -> Vec<Component>;
}

#[macro_export]
macro_rules! print_linked_data_schema_for {
  ( $x:ty ) => {
    let schema = <$x>::shacl();
    let mut shacl_writer = ShaclWriter::<SRDFGraph>::default();

    shacl_writer.write(&schema).unwrap();

    let mut cursor = Cursor::new(Vec::new());

    shacl_writer
      .serialize(&RDFFormat::Turtle, &mut cursor)
      .unwrap();

    let content = cursor.into_inner();
    let s = String::from_utf8(content).unwrap();
    println!("{}", s);
  };
}
