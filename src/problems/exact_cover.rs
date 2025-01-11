use std::collections::{HashMap, HashSet};

#[derive(Debug, Default)]
pub struct ExactCover {
    columns: HashMap<usize, Column>,
    nodes: Vec<Node>,
}

#[derive(Debug)]
struct Column {
    constraint: usize,
    header_index: usize,
    len: usize,
    is_covered: bool,
}

#[derive(Debug)]
struct Node {
    name: u8,
    constraint: usize,
    is_header: bool,

    up_index: usize,
    down_index: usize,
    left_index: usize,
    right_index: usize,
}

impl ExactCover {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn add(&mut self, name: u8, constraints: &[usize]) {
        for constraint in constraints {
            if !self.columns.contains_key(constraint) {
                self.add_column(*constraint);
            }
        }

        for (i, &constraint) in constraints.iter().enumerate() {
            let node_index = self.nodes.len();

            let left_index = if i == 0 {
                node_index + constraints.len() - 1
            } else {
                node_index - 1
            };

            let right_index = if i == constraints.len() - 1 {
                node_index - i
            } else {
                node_index + 1
            };

            let node = Node {
                name,
                constraint,
                is_header: false,
                up_index: node_index,
                down_index: node_index,
                left_index,
                right_index,
            };

            self.append_node(constraint, node);
        }
    }

    pub fn solve(&mut self) -> Vec<u8> {
        ExactCoverSolver::new(self).solve()
    }

    fn add_column(&mut self, constraint: usize) {
        let header_index = self.nodes.len();

        let column = Column {
            constraint,
            header_index,
            len: 0,
            is_covered: false,
        };

        let header = Node {
            name: 0,
            constraint,
            is_header: true,
            up_index: header_index,
            down_index: header_index,
            left_index: header_index,
            right_index: header_index,
        };

        self.columns.insert(constraint, column);
        self.nodes.push(header);
    }

    fn append_node(&mut self, constraint: usize, mut node: Node) {
        let column = self.column_mut(constraint);
        column.len += 1;

        let node_index = node.down_index;

        //
        let header_index = column.header_index;
        let header_node = self.node_mut(header_index);

        //
        let last_node_index = header_node.up_index;

        //
        header_node.up_index = node_index;

        //
        let last_node = self.node_mut(last_node_index);
        last_node.down_index = node_index;

        //
        node.up_index = last_node_index;
        node.down_index = header_index;

        self.nodes.push(node);
    }

    fn sorted_columns(&mut self) -> Vec<usize> {
        let mut columns: Vec<&Column> = self
            .columns
            .values()
            .filter(|column| !column.is_covered)
            .collect();

        columns.sort_unstable_by_key(|column| (column.len, column.constraint));
        columns.iter().map(|column| column.constraint).collect()
    }

    fn column(&self, constraint: usize) -> &Column {
        self.columns.get(&constraint).unwrap()
    }

    fn column_mut(&mut self, constraint: usize) -> &mut Column {
        self.columns.get_mut(&constraint).unwrap()
    }

    fn node(&self, node_index: usize) -> &Node {
        self.nodes.get(node_index).unwrap()
    }

    fn node_mut(&mut self, node_index: usize) -> &mut Node {
        self.nodes.get_mut(node_index).unwrap()
    }
}

struct ExactCoverSolver<'a> {
    problem: &'a mut ExactCover,
    num_covered_nodes: usize,
}

impl<'a> ExactCoverSolver<'a> {
    pub fn new(problem: &'a mut ExactCover) -> Self {
        Self {
            problem,
            num_covered_nodes: 0,
        }
    }

    pub fn solve(&mut self) -> Vec<u8> {
        let mut solution = Vec::new();

        self.solve_internal(&mut solution);
        self.reset();

        solution
    }

    fn solve_internal(&mut self, solution: &mut Vec<u8>) -> bool {
        if self.is_solved() {
            return true;
        }

        for constraint in self.problem.sorted_columns() {
            let column = self.problem.column(constraint);

            // no solution exists in this branch
            if column.len == 0 {
                return false;
            }

            for column_index in self.indexes_from(column.header_index, Direction::Column) {
                solution.push(self.problem.node(column_index).name);

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

                if self.solve_internal(solution) {
                    return true;
                }

                solution.pop();

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

    fn reset(&mut self) {}
}

enum Direction {
    Column,
    Row,
}
