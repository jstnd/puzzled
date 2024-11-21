use std::collections::HashMap;

#[derive(Default)]
pub struct ExactCoverProblem {
    pub(crate) columns: HashMap<usize, ExactCoverColumn>,
    pub(crate) nodes: Vec<ExactCoverNode>,
}

impl ExactCoverProblem {
    pub fn add_constraint(&mut self, name: u8, elements: &[usize]) {
        for element in elements {
            if !self.columns.contains_key(element) {
                self.add_column(*element);
            }
        }

        for (element_index, &element) in elements.iter().enumerate() {
            let node_index = self.nodes.len();

            let left_index = if element_index == 0 {
                node_index + elements.len() - 1
            } else {
                node_index - 1
            };

            let right_index = if element_index == elements.len() - 1 {
                node_index - element_index
            } else {
                node_index + 1
            };

            let node = ExactCoverNode {
                name,
                element,
                is_header: false,
                up_index: node_index,
                down_index: node_index,
                left_index,
                right_index,
            };

            self.append_node(element, node);
        }
    }

    fn add_column(&mut self, element: usize) {
        let header_index = self.nodes.len();

        let column = ExactCoverColumn {
            element,
            header_index,
            len: 0,
            is_covered: false,
        };

        let header = ExactCoverNode {
            name: 0,
            element,
            is_header: true,
            up_index: header_index,
            down_index: header_index,
            left_index: header_index,
            right_index: header_index,
        };

        self.columns.insert(element, column);
        self.nodes.push(header);
    }

    fn append_node(&mut self, element: usize, mut node: ExactCoverNode) {
        let column = self.columns.get_mut(&element).unwrap();
        column.len += 1;

        let node_index = node.down_index;

        //
        let header_node = self.nodes.get_mut(column.header_index).unwrap();

        //
        let last_node_index = header_node.up_index;

        //
        header_node.up_index = node_index;

        //
        let last_node = self.nodes.get_mut(last_node_index).unwrap();
        last_node.down_index = node_index;

        //
        node.up_index = last_node_index;
        node.down_index = column.header_index;

        self.nodes.push(node);
    }
}

pub(crate) struct ExactCoverColumn {
    pub(crate) element: usize,
    pub(crate) header_index: usize,
    pub(crate) len: usize,
    pub(crate) is_covered: bool,
}

pub(crate) struct ExactCoverNode {
    pub(crate) name: u8,
    pub(crate) element: usize,
    pub(crate) is_header: bool,

    pub(crate) up_index: usize,
    pub(crate) down_index: usize,
    pub(crate) left_index: usize,
    pub(crate) right_index: usize,
}
