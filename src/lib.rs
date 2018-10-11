extern crate sxd_document;

use std::collections::HashMap;

use sxd_document::{
    parser,
    Package,
};
use sxd_document::dom::{
    Document,
    Root,
    ChildOfRoot,
    Element,
    ChildOfElement,
};

#[derive(Debug)]
pub struct Program {
    pub groups: Vec<StatementBody>
}

#[derive(PartialEq, Debug)]
pub struct StatementBody {
    pub blocks: Vec<Block>
}

#[derive(PartialEq, Debug)]
pub struct Block {
    pub block_type: String,
    pub id: String,
    pub fields: HashMap<String, FieldValue>,
    pub statements: HashMap<String, StatementBody>,
}

#[derive(PartialEq, Debug)]
pub enum FieldValue {
    SimpleField(String),
    ExpressionField(Block),
}

impl Program {
    pub fn new() -> Self {
        Self {
            groups: Vec::new()
        }
    }
}

impl StatementBody {
    fn new(first_block: Option<Element>) -> Self {
        let mut blocks = Vec::new();
        if let Some(el) = first_block {
            // Create each block, put them into the statement body
            let mut block_el: Element;
            block_el = el;
            loop {
                blocks.push(Block::new(block_el));
                if let Some(next_block) = get_next_block_element(block_el) {
                    block_el = next_block;
                } else {
                    break;
                }
            }
        }
        Self {
            blocks
        }
    }
}

impl Block {
    fn new(block_el: Element) -> Self {
        let mut block = Self {
            block_type: "".to_string(),
            id: "".to_string(),
            fields: HashMap::new(),
            statements: HashMap::new()
        };

        for attribute in block_el.attributes().iter() {
            let name = attribute.name().local_part();
            let value = attribute.value().to_string();
            match name {
                "type" => { block.block_type = value; },
                "id" => { block.id = value; },
                _ => {}
            }
        }

        for child in block_el.children().iter() {
            if let &ChildOfElement::Element(child_el) = child {
                let child_name = child_el.name().local_part();
                match child_name {
                    "statement" => {
                        let statement_el = child_el;
                        let statement_name = get_attribute(statement_el, "name").unwrap();
                        let statement_body = StatementBody::new(get_first_child_element(statement_el));
                        block.statements.insert(statement_name, statement_body);
                    },
                    "field" => {
                        let field_el = child_el;
                        let field_name = get_attribute(field_el, "name").unwrap();
                        let field_value = FieldValue::new(field_el);
                        block.fields.insert(field_name, field_value);
                    },
                    _ => {}
                }
            }
        }

        block
    }
}

impl FieldValue {
    fn new(field_el: Element) -> Self {
        for child in field_el.children().iter() {
            match child {
                &ChildOfElement::Text(text_node) => {
                    let value = text_node.text().to_string();
                    return FieldValue::SimpleField(value);
                },
                _ => panic!("TODO: Implement expression fields")
            }
        }
        panic!("Expected child nodes for field");
    }
}

// Utilities for creating Blockly data structures

pub fn program_from_xml(xml: &str) -> Program {
    let mut program = Program::new();

    let package: Package = parser::parse(xml).expect("Failed to parse XML!");
    let document: Document = package.as_document();

    let xml_element = get_xml_element(document);

    for child in xml_element.children().iter() {
        if let &ChildOfElement::Element(el) = child {
            let element_name = el.name().local_part();
            match element_name {
                "block" => {
                    program.groups.push(StatementBody::new(Some(el)));
                },
                // TODO: handle `variables`
                _ => {}
            }
        }
    }

    program
}

fn get_next_block_element(block_el: Element) -> Option<Element> {
    let next_el: Option<Element> = {
        let mut found: Option<Element> = None;
        for child in block_el.children().iter() {
            if let &ChildOfElement::Element(el) = child {
                if el.name().local_part() == "next" {
                    found = Some(el);
                    break;
                }
            }
        }
        found
    };

    if let Some(next_el) = next_el {
        for child in next_el.children().iter() {
            if let &ChildOfElement::Element(el) = child {
                if el.name().local_part() == "block" {
                    return Some(el);
                }
            }
        }
    }

    None
}

// General DOM utilities

fn get_xml_element(document: Document) -> Element {
    let root: Root = document.root();
    let root_children = root.children();
    for child in root_children.iter() {
        if let &ChildOfRoot::Element(el) = child {
            let element_name = el.name().local_part();
            if element_name == "xml" {
                return el;
            }
        }
    }
    panic!("Cannot find xml element!");
}

fn get_first_child_element(element: Element) -> Option<Element> {
    for child in element.children().iter() {
        if let &ChildOfElement::Element(el) = child {
            return Some(el);
        }
    }
    None
}

fn get_attribute(element: Element, attribute_name: &str) -> Option<String> {
    for attribute in element.attributes().iter() {
        let name = attribute.name().local_part();
        let value = attribute.value().to_string();
        if name == attribute_name {
            return Some(value);
        }
    }
    None
}


#[cfg(test)]
mod test {
    use super::*;

    fn get_fragment_root(package: &Package) -> Option<Element> {
        let document: Document = package.as_document();
        let mut root_element: Option<Element> = None;
        for child in document.root().children().iter() {
            if let &ChildOfRoot::Element(el) = child {
                root_element = Some(el);
                break;
            }
        }
        root_element
    }

    #[test]
    fn test_new_block() {
        let xml: &str = r#"
            <block type="inner_loop" id="]Lb|t?wfd#;s)[llJx8Y">
                <field name="COUNT">3</field>
                <statement name="BODY">
                </statement>
            </block>
        "#;
        let fragment: Package = parser::parse(xml).expect("Failed to parse XML!");
        let root_element = get_fragment_root(&fragment).unwrap();

        let block = Block::new(root_element);
        assert_eq!(block.block_type, "inner_loop");
        assert_eq!(block.id, "]Lb|t?wfd#;s)[llJx8Y");
        let count_field = block.fields.get("COUNT");
        assert!(count_field.is_some());
        assert_eq!(count_field.unwrap(), &FieldValue::SimpleField("3".to_string()));
    }

    #[test]
    fn test_get_next_block_element() {
        let xml: &str = r#"
            <block type="led_on" id="^3xb.m4E9i0;3$R10(=5">
                <field name="TIME">300</field>
                <next>
                    <block type="led_off" id="HX4*sB9=gbJtq$Y{ke6b">
                        <field name="TIME">100</field>
                    </block>
                </next>
            </block>
        "#;
        let fragment: Package = parser::parse(xml).expect("Failed to parse XML!");
        let root_element = get_fragment_root(&fragment).unwrap();

        let next_block = get_next_block_element(root_element);
        assert!(next_block.is_some());
        let next_block_unwrapped = next_block.unwrap();
        assert_eq!(get_attribute(next_block_unwrapped, "type"), Some("led_off".to_string()));
        assert_eq!(get_attribute(next_block_unwrapped, "id"), Some("HX4*sB9=gbJtq$Y{ke6b".to_string()));
    }

    #[test]
    fn test_program_from_xml_advanced() {
        let xml: &str = r#"
            <xml xmlns="http://www.w3.org/1999/xhtml">
                <variables></variables>
                <block type="main_loop" id="[.)/fqUYv92(mzb{?:~u" deletable="false" movable="false" x="50" y="50">
                    <statement name="BODY">
                        <block type="inner_loop" id="]Lb|t?wfd#;s)[llJx8Y">
                            <field name="COUNT">3</field>
                            <statement name="BODY">
                                <block type="led_on" id="^3xb.m4E9i0;3$R10(=5">
                                    <field name="TIME">300</field>
                                    <next>
                                        <block type="led_off" id="HX4*sB9=gbJtq$Y{ke6b">
                                            <field name="TIME">100</field>
                                        </block>
                                    </next>
                                </block>
                            </statement>
                            <next>
                                <block type="led_on" id="kB~f~7W`wkGa0i4z3mHw">
                                    <field name="TIME">100</field>
                                    <next>
                                        <block type="led_off" id="$fdlZB)btzA8YtB/!xz`">
                                            <field name="TIME">100</field>
                                        </block>
                                    </next>
                                </block>
                            </next>
                        </block>
                    </statement>
                </block>
            </xml>
        "#;

        let program: Program = program_from_xml(xml);
        assert_eq!(program.groups.len(), 1);

        let group = program.groups.get(0).unwrap();
        assert_eq!(group.blocks.len(), 1);

        let main_loop_block = group.blocks.get(0).unwrap();
        assert_eq!(main_loop_block.block_type, "main_loop");
        assert_eq!(main_loop_block.id, "[.)/fqUYv92(mzb{?:~u");

        let main_loop_statements = &main_loop_block.statements;
        assert_eq!(main_loop_statements.len(), 1);
        assert!(main_loop_statements.contains_key("BODY"));

        let main_loop_body = main_loop_statements.get("BODY");
        let main_loop_body_statement = main_loop_body.as_ref().unwrap();
        assert_eq!(main_loop_body_statement.blocks.len(), 3);

        let inner_loop_block = main_loop_body_statement.blocks.get(0).unwrap();
        assert_eq!(inner_loop_block.block_type, "inner_loop");
        assert_eq!(inner_loop_block.id, "]Lb|t?wfd#;s)[llJx8Y");
        assert_eq!(inner_loop_block.fields.get("COUNT"), Some(&FieldValue::SimpleField("3".to_string())));

        let inner_loop_statement_maybe = inner_loop_block.statements.get("BODY");
        assert!(inner_loop_statement_maybe.is_some());
        let inner_loop_statement = inner_loop_statement_maybe.unwrap();
        assert_eq!(inner_loop_statement.blocks.len(), 2);

        let led_on_block = inner_loop_statement.blocks.get(0).unwrap();
        assert_eq!(led_on_block.block_type, "led_on");
        assert_eq!(led_on_block.id, "^3xb.m4E9i0;3$R10(=5");
        assert_eq!(led_on_block.fields.get("TIME"), Some(&FieldValue::SimpleField("300".to_string())));

        let led_off_block = inner_loop_statement.blocks.get(1).unwrap();
        assert_eq!(led_off_block.block_type, "led_off");
        assert_eq!(led_off_block.id, "HX4*sB9=gbJtq$Y{ke6b");
    }
}
