use std::collections::HashSet;

use super::problem::{ExactCoverColumn, ExactCoverProblem};

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
        let column_constraints: Vec<usize> =
            columns.iter().map(|column| column.constraint).collect();

        for constraint in column_constraints {
            let column = self.problem.column(constraint);

            // no solution exists in this branch
            if column.len == 0 {
                return false;
            }

            for column_index in self.indexes_from(column.header_index, Direction::Column) {
                self.solution.push(self.problem.node(column_index).name);

                let mut covered_columns = Vec::new();
                let nodes_to_cover = self.nodes_to_cover(column_index);

                for row_index in self.indexes_from(column_index, Direction::Row) {
                    let constraint = self.problem.node(row_index).constraint;
                    let column = self.problem.column_mut(constraint);
                    column.is_covered = true;
                    covered_columns.push(constraint);
                }

                for node_index in nodes_to_cover.iter() {
                    self.cover(*node_index);
                }

                if self.search() {
                    return true;
                }

                self.solution.pop();

                // uncover everything
                for constraint in covered_columns {
                    self.problem.column_mut(constraint).is_covered = false;
                }

                for node_index in nodes_to_cover.iter().rev() {
                    self.uncover(*node_index);
                }
            }
        }

        false
    }

    fn nodes_to_cover(&self, start_index: usize) -> Vec<usize> {
        let mut node_indexes = HashSet::new();

        for row_index in self.indexes_from(start_index, Direction::Row) {
            for column_index in self.indexes_from(row_index, Direction::Column) {
                node_indexes.insert(column_index);
                node_indexes.extend(self.indexes_from(column_index, Direction::Row));
            }
        }

        Vec::from_iter(node_indexes)
    }

    fn indexes_from(&self, start_index: usize, direction: Direction) -> Vec<usize> {
        let mut node_indexes = Vec::new();
        let mut current_node_index = start_index;
        let mut current_node = self.problem.node(current_node_index);

        loop {
            if !current_node.is_header {
                node_indexes.push(current_node_index);
            }

            current_node_index = match direction {
                Direction::Column => current_node.down_index,
                Direction::Row => current_node.right_index,
            };

            current_node = self.problem.node(current_node_index);

            if current_node_index == start_index {
                break;
            }
        }

        node_indexes
    }

    fn cover(&mut self, node_index: usize) {
        self.num_covered_nodes += 1;

        let node = self.problem.node(node_index);
        let (constraint, up_index, down_index, left_index, right_index) = (
            node.constraint,
            node.up_index,
            node.down_index,
            node.left_index,
            node.right_index,
        );

        self.problem.column_mut(constraint).len -= 1;
        self.problem.node_mut(up_index).down_index = down_index;
        self.problem.node_mut(down_index).up_index = up_index;
        self.problem.node_mut(left_index).right_index = right_index;
        self.problem.node_mut(right_index).left_index = left_index;
    }

    fn uncover(&mut self, node_index: usize) {
        self.num_covered_nodes -= 1;

        let node = self.problem.node(node_index);
        let (constraint, up_index, down_index, left_index, right_index) = (
            node.constraint,
            node.up_index,
            node.down_index,
            node.left_index,
            node.right_index,
        );

        self.problem.column_mut(constraint).len += 1;
        self.problem.node_mut(up_index).down_index = node_index;
        self.problem.node_mut(down_index).up_index = node_index;
        self.problem.node_mut(left_index).right_index = node_index;
        self.problem.node_mut(right_index).left_index = node_index;
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

enum Direction {
    Column,
    Row,
}
