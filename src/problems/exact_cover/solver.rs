use std::collections::HashSet;

use super::problem::{ExactCoverColumn, ExactCoverDirection, ExactCoverProblem};

pub struct ExactCoverSolver {
    problem: ExactCoverProblem,

    num_covered_nodes: usize,
    pub solution: Vec<u8>,
}

impl ExactCoverSolver {
    pub fn new(problem: ExactCoverProblem) -> Self {
        Self {
            problem,
            solution: Vec::new(),
            num_covered_nodes: 0,
        }
    }

    pub fn solve(&mut self) {
        self.search();
    }

    fn search(&mut self) -> bool {
        if self.is_solved() {
            return true;
        }

        let mut columns: Vec<&ExactCoverColumn> = self
            .problem
            .columns
            .values()
            .filter(|column| !column.is_covered)
            .collect();

        columns.sort_by_key(|column| column.len);
        let column_elements: Vec<usize> = columns.iter().map(|column| column.element).collect();

        for element in column_elements {
            let column = self.problem.columns.get(&element).unwrap();

            // no solution exists in this branch
            if column.len == 0 {
                return false;
            }

            for column_index in self.get_indexes(column.header_index, ExactCoverDirection::Column) {
                self.solution
                    .push(self.problem.nodes.get(column_index).unwrap().name);

                let mut covered_columns = Vec::new();
                let nodes_to_cover = self.get_nodes_to_cover(column_index);

                for row_index in self.get_indexes(column_index, ExactCoverDirection::Row) {
                    let element = self.problem.nodes.get(row_index).unwrap().element;
                    let column = self.problem.columns.get_mut(&element).unwrap();
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
                    self.problem.columns.get_mut(&element).unwrap().is_covered = false;
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
        let mut current_node = self.problem.nodes.get(current_node_index).unwrap();

        loop {
            if !current_node.is_header {
                node_indexes.push(current_node_index);
            }

            current_node_index = match direction {
                ExactCoverDirection::Column => current_node.down_index,
                ExactCoverDirection::Row => current_node.right_index,
            };

            current_node = self.problem.nodes.get(current_node_index).unwrap();

            if current_node_index == start_index {
                break;
            }
        }

        node_indexes
    }

    fn cover_node(&mut self, node_index: usize) {
        self.num_covered_nodes += 1;

        let node = self.problem.nodes.get(node_index).unwrap();
        let (element, up_index, down_index, left_index, right_index) = (
            node.element,
            node.up_index,
            node.down_index,
            node.left_index,
            node.right_index,
        );

        self.problem.columns.get_mut(&element).unwrap().len -= 1;
        self.problem.nodes.get_mut(up_index).unwrap().down_index = down_index;
        self.problem.nodes.get_mut(down_index).unwrap().up_index = up_index;
        self.problem.nodes.get_mut(left_index).unwrap().right_index = right_index;
        self.problem.nodes.get_mut(right_index).unwrap().left_index = left_index;
    }

    fn uncover_node(&mut self, node_index: usize) {
        self.num_covered_nodes -= 1;

        let node = self.problem.nodes.get(node_index).unwrap();
        let (element, up_index, down_index, left_index, right_index) = (
            node.element,
            node.up_index,
            node.down_index,
            node.left_index,
            node.right_index,
        );

        self.problem.columns.get_mut(&element).unwrap().len += 1;
        self.problem.nodes.get_mut(up_index).unwrap().down_index = node_index;
        self.problem.nodes.get_mut(down_index).unwrap().up_index = node_index;
        self.problem.nodes.get_mut(left_index).unwrap().right_index = node_index;
        self.problem.nodes.get_mut(right_index).unwrap().left_index = node_index;
    }

    fn is_solved(&self) -> bool {
        self.num_covered_nodes == self.problem.nodes.len() - self.problem.columns.len()
            && self
                .problem
                .columns
                .values()
                .all(|column| column.is_covered)
    }
}
