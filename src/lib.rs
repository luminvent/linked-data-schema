mod linked_data_schema_field_visitor;

pub use linked_data_schema_derive::LinkedDataSchema;
pub use linked_data_schema_field_visitor::LinkedDataSchemaFieldVisitor;
use shacl::ast::{ASTComponent, ASTSchema};

pub mod reexports {
  pub use iri_s;
  pub use prefixmap;
  pub use rudof_rdf;
  pub use shacl;
  pub use uuid;
}

pub trait LinkedDataSchema {
  fn shacl() -> ASTSchema;

  fn components() -> Vec<ASTComponent>;
}

#[macro_export]
macro_rules! print_linked_data_schema_for {
  ( $x:ty ) => {
    let schema = <$x>::shacl();
    {
      use ::linked_data_schema::reexports::rudof_rdf::rdf_core::RDFFormat::Turtle;
      use ::linked_data_schema::reexports::rudof_rdf::rdf_impl::InMemoryGraph;
      use ::linked_data_schema::reexports::shacl::ir::IRSchema;
      use ::linked_data_schema::reexports::shacl::rdf::ShaclWriter;

      let mut shacl_writer = ShaclWriter::<InMemoryGraph>::default();

      println!("{:#?}", schema);

      let ir_schema = IRSchema::try_from(schema).unwrap();

      shacl_writer.register(&ir_schema).unwrap();

      let mut cursor = std::io::Cursor::new(Vec::new());

      shacl_writer.serialize(&Turtle, &mut cursor).unwrap();

      let content = cursor.into_inner();
      let s = String::from_utf8(content).unwrap();
      println!("{}", s);
    }
  };
}
