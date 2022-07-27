use {
    crate::*,
    std::collections::BinaryHeap,
};

/// Find a short path between start and goal using A*.
///
/// The returned path contains the goal but not the start.
pub fn find_astar(maze: &Maze, start: Pos, goal: Pos) -> Option<Vec<Pos>> {
    let dim = maze.dim;

    // nodes already evaluated, we know they're not interesting
    let mut closed_set = PosSet::new(dim, false);

    // node immediately preceding on the cheapest known path from start
    let mut came_from: PosMap<Pos> = PosMap::new(dim, Pos::new(0, 0));

    // g_score is the cost of the cheapest path from start to a pos
    let mut g_score: PosMap<i32> = PosMap::new(dim, i32::MAX);
    g_score.set(start, 0);

    // the nodes from which we may expand
    let mut open_set: BinaryHeap<ValuedPos> = BinaryHeap::new();
    open_set.push(ValuedPos::from(start, 0));

    while let Some(mut current) = open_set.pop().map(|vp| vp.pos) {
        closed_set.set(current, true);
        let neighbours = maze.enterable_neighbours(current);
        for neighbour in &neighbours {
            if Pos::sides(*neighbour, goal) {
                let mut path = vec![*neighbour];
                while current != start {
                    path.push(current);
                    current = came_from.get(current);
                }
                path.reverse();
                return Some(path);
            }
            if closed_set.get(*neighbour) {
                continue;
            }
            let tentative_g_score = g_score.get(current) + 1;
            let previous_g_score = g_score.get(*neighbour);
            if tentative_g_score < previous_g_score {
                came_from.set(*neighbour, current);
                g_score.set(*neighbour, tentative_g_score);
                let new_f_score =
                    tentative_g_score + 2 * Pos::euclidian_distance(*neighbour, goal) as i32;
                open_set.push(ValuedPos::from(*neighbour, new_f_score));
            }
        }
    }

    // open_set is empty, there's no path
    None
}
