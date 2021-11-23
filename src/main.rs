use rand::seq::SliceRandom;
use sixtyfps::{Model, ModelHandle, VecModel};
use std::rc::Rc;
use std::time::Duration;

sixtyfps::include_modules!();

fn main() {
    let main_window = MainWindow::new();
    let main_window_weak = main_window.as_weak();

    // Initialize the board in order to double the tiles
    let mut tiles: Vec<TileData> = main_window_weak
        .unwrap()
        .get_memory_tiles()
        .iter()
        .collect();
    tiles.extend(tiles.clone());

    // This is the singular tiles_model that needs to be used throughout the
    // file
    let tiles_model = Rc::new(VecModel::from(tiles.clone()));

    main_window_weak
        .unwrap()
        .set_memory_tiles(ModelHandle::new(tiles_model.clone()));

    reset_board(&main_window_weak, tiles_model.clone());

    // We need to create a clone because this value with be dropped because of the move closure
    let tiles_model_clone = tiles_model.clone();
    let main_window_weak_clone = main_window_weak.clone();

    main_window.on_check_if_pair_solved(move || {
        let mut flipped_tiles = tiles_model_clone
            .iter()
            .enumerate()
            .filter(|(_, tile)| tile.image_visible && !tile.solved);

        if let (Some((t1_idx, mut t1)), Some((t2_idx, mut t2))) =
            (flipped_tiles.next(), flipped_tiles.next())
        {
            let is_pair_solved = t1 == t2;
            if is_pair_solved {
                t1.solved = true;
                tiles_model_clone.set_row_data(t1_idx, t1);
                t2.solved = true;
                tiles_model_clone.set_row_data(t2_idx, t2);
            } else {
                let main_window = main_window_weak_clone.unwrap();
                main_window.set_disable_tiles(true);
                let tiles_model = tiles_model_clone.clone();
                sixtyfps::Timer::single_shot(Duration::from_secs(1), move || {
                    main_window.set_disable_tiles(false);
                    t1.image_visible = false;
                    tiles_model.set_row_data(t1_idx, t1);
                    t2.image_visible = false;
                    tiles_model.set_row_data(t2_idx, t2);
                });
            }
        }
    });

    // Clone again because of move
    let main_window_weak_clone = main_window_weak.clone();

    main_window.on_check_if_game_finished(move || {
        let count_unfinished = main_window_weak_clone
            .unwrap()
            .get_memory_tiles()
            .iter()
            .filter(|tile| !tile.solved)
            .count();
        if count_unfinished == 0 {
            // Cloning again
            let main_window_weak_clone = main_window_weak.clone();
            let tiles_model = tiles_model.clone();
            sixtyfps::Timer::single_shot(Duration::from_secs(1), move || {
                for (idx, mut tile) in main_window_weak_clone
                    .unwrap()
                    .get_memory_tiles()
                    .iter()
                    .enumerate()
                {
                    tile.image_visible = false;
                    tile.solved = false;
                    tiles_model.set_row_data(idx, tile);
                }
                // No clone needed in here because it is fine for this one to be consumed because
                // we don't use it after this
                sixtyfps::Timer::single_shot(Duration::from_millis(250), move || {
                    reset_board(&main_window_weak_clone, tiles_model.clone());
                });
            });
        }
    });

    main_window.run();
}

// The main window is needed because it should be the only instance and we want to use the same
// tiles model because that is what contians the current state of all the board peices. Creating a
// new one loses those states
fn reset_board(main_window: &sixtyfps::Weak<MainWindow>, tiles_model: Rc<VecModel<TileData>>) {
    // Get the tiles, double them and shuffle them
    let mut tiles: Vec<TileData> = main_window.unwrap().get_memory_tiles().iter().collect();
    let mut rng = rand::thread_rng();
    tiles.shuffle(&mut rng);

    for (idx, mut tile) in tiles.iter_mut().enumerate() {
        tile.image_visible = false;
        tile.solved = false;
        tiles_model.set_row_data(idx, tile.clone());
    }
    main_window
        .unwrap()
        .set_memory_tiles(ModelHandle::new(tiles_model));
}
