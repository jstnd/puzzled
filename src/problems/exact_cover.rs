use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub struct ExactCover {
    columns: HashMap<usize, ExactCoverColumn>,
    nodes: Vec<ExactCoverNode>,
    num_covered_nodes: usize,

    pub solution: Vec<u8>,
}

struct ExactCoverColumn {
    element: usize,

    header_index: usize,
    len: usize,

    is_covered: bool,
}

struct ExactCoverNode {
    name: u8,
    element: usize,
    is_header: bool,

    up_index: usize,
    down_index: usize,
    left_index: usize,
    right_index: usize,
}

enum ExactCoverDirection {
    Column,
    Row,
}

impl ExactCover {
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

impl ExactCover {
    pub fn solve(&mut self) {
        self.search();
    }

    fn search(&mut self) -> bool {
        if self.is_solved() {
            return true;
        }

        let mut columns: Vec<&ExactCoverColumn> = self
            .columns
            .values()
            .filter(|column| !column.is_covered)
            .collect();

        columns.sort_by_key(|column| column.len);
        let column_elements: Vec<usize> = columns.iter().map(|column| column.element).collect();

        for element in column_elements {
            let column = self.columns.get(&element).unwrap();

            // no solution exists in this branch
            if column.len == 0 {
                return false;
            }

            for column_index in self.get_indexes(column.header_index, ExactCoverDirection::Column) {
                self.solution
                    .push(self.nodes.get(column_index).unwrap().name);

                let mut covered_columns = Vec::new();
                let nodes_to_cover = self.get_nodes_to_cover(column_index);

                for row_index in self.get_indexes(column_index, ExactCoverDirection::Row) {
                    let element = self.nodes.get(row_index).unwrap().element;
                    let column = self.columns.get_mut(&element).unwrap();
                    column.is_covered = true;
                    covered_columns.push(element);
                }

                for node_index in nodes_to_cover.iter() {
                    self.cover_node(*node_index);
                }

                if self.search() {
                    return true;
                }

                self.solution.pop();

                // uncover everything
                for element in covered_columns {
                    self.columns.get_mut(&element).unwrap().is_covered = false;
                }

                for node_index in nodes_to_cover.iter().rev() {
                    self.uncover_node(*node_index);
                }
            }
        }

        false
    }

    fn get_nodes_to_cover(&self, start_index: usize) -> Vec<usize> {
        let mut node_indexes = HashSet::new();

        for row_index in self.get_indexes(start_index, ExactCoverDirection::Row) {
            for column_index in self.get_indexes(row_index, ExactCoverDirection::Column) {
                node_indexes.insert(column_index);
                node_indexes.extend(self.get_indexes(column_index, ExactCoverDirection::Row));
            }
        }

        Vec::from_iter(node_indexes)
    }

    fn get_indexes(&self, start_index: usize, direction: ExactCoverDirection) -> Vec<usize> {
        let mut node_indexes = Vec::new();
        let mut current_node_index = start_index;
        let mut current_node = self.nodes.get(current_node_index).unwrap();

        loop {
            if !current_node.is_header {
                node_indexes.push(current_node_index);
            }

            current_node_index = match direction {
                ExactCoverDirection::Column => current_node.down_index,
                ExactCoverDirection::Row => current_node.right_index,
            };

            current_node = self.nodes.get(current_node_index).unwrap();

            if current_node_index == start_index {
                break;
            }
        }

        node_indexes
    }

    fn cover_node(&mut self, node_index: usize) {
        self.num_covered_nodes += 1;

        let node = self.nodes.get(node_index).unwrap();
        let (element, up_index, down_index, left_index, right_index) = (
            node.element,
            node.up_index,
            node.down_index,
            node.left_index,
            node.right_index,
        );

        self.columns.get_mut(&element).unwrap().len -= 1;
        self.nodes.get_mut(up_index).unwrap().down_index = down_index;
        self.nodes.get_mut(down_index).unwrap().up_index = up_index;
        self.nodes.get_mut(left_index).unwrap().right_index = right_index;
        self.nodes.get_mut(right_index).unwrap().left_index = left_index;
    }

    fn uncover_node(&mut self, node_index: usize) {
        self.num_covered_nodes -= 1;

        let node = self.nodes.get(node_index).unwrap();
        let (element, up_index, down_index, left_index, right_index) = (
            node.element,
            node.up_index,
            node.down_index,
            node.left_index,
            node.right_index,
        );

        self.columns.get_mut(&element).unwrap().len += 1;
        self.nodes.get_mut(up_index).unwrap().down_index = node_index;
        self.nodes.get_mut(down_index).unwrap().up_index = node_index;
        self.nodes.get_mut(left_index).unwrap().right_index = node_index;
        self.nodes.get_mut(right_index).unwrap().left_index = node_index;
    }

    fn is_solved(&self) -> bool {
        self.num_covered_nodes == self.nodes.len() - self.columns.len()
            && self.columns.values().all(|column| column.is_covered)
    }
}
