mod linked_data_schema_field_visitor;
#[cfg(feature = "utoipa")]
mod linked_data_to_schema_field;

pub use linked_data_schema_derive::LinkedDataSchema;
#[cfg(feature = "utoipa")]
pub use linked_data_schema_derive::LinkedDataToSchema;
pub use linked_data_schema_field_visitor::LinkedDataSchemaFieldVisitor;
#[cfg(feature = "utoipa")]
pub use linked_data_to_schema_field::LinkedDataToSchemaField;
use shacl::ast::{ASTComponent, ASTSchema};

pub mod reexports {
  pub use iri_s;
  pub use prefixmap;
  pub use rudof_rdf;
  #[cfg(feature = "utoipa")]
  pub use serde_json;
  pub use shacl;
  #[cfg(feature = "utoipa")]
  pub use utoipa;
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
