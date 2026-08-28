use imgui_app::dear_imgui_rs::{Ui, MouseButton};
use ksmap::{screen_map::ScreenMap, partition::Partition};
use libks::ScreenCoord;
use rustc_hash::FxHashMap;

pub struct MapState {
    pub top_left: ScreenCoord,
    pub is_dragging: bool,
    pub zoom_level: i32,
    pub bias: (f32, f32),
    pub aspect_ratio: f32,
    pub prev_geom: Option<MapGeometry>,
    pub selected_screen: Option<ScreenCoord>,
}

impl Default for MapState {
    fn default() -> Self {
        Self {
            top_left: (1000, 1000),
            is_dragging: false,
            zoom_level: 6,
            bias: (0.0, 0.0),
            aspect_ratio: 1.0,
            prev_geom: None,
            selected_screen: None,
        }
    }
}

const MAP_COLORS: &'static [[f32; 4]] = &[
    [0.114, 0.169, 0.325, 1.0],
    [0.494, 0.145, 0.325, 1.0],
    [0.000, 0.529, 0.318, 1.0],
    [0.671, 0.322, 0.212, 1.0],
    [0.373, 0.341, 0.310, 1.0],
    [0.761, 0.765, 0.780, 1.0],
    [1.000, 0.945, 0.910, 1.0],
    [1.000, 0.000, 0.302, 1.0],
    [1.000, 0.639, 0.000, 1.0],
    [1.000, 0.925, 0.153, 1.0],
    [0.000, 0.894, 0.212, 1.0],
    [0.161, 0.678, 1.000, 1.0],
    [0.514, 0.463, 0.612, 1.0],
    [1.000, 0.467, 0.659, 1.0],
    [1.000, 0.800, 0.667, 1.0],
    [0.000, 0.000, 0.000, 1.0],
    [0.161, 0.094, 0.078, 1.0],
    [0.067, 0.114, 0.208, 1.0],
    [0.259, 0.129, 0.212, 1.0],
    [0.071, 0.325, 0.349, 1.0],
    [0.455, 0.184, 0.161, 1.0],
    [0.286, 0.200, 0.231, 1.0],
    [0.635, 0.533, 0.475, 1.0],
    [0.953, 0.937, 0.490, 1.0],
    [0.745, 0.071, 0.314, 1.0],
    [1.000, 0.424, 0.141, 1.0],
    [0.659, 0.906, 0.180, 1.0],
    [0.000, 0.710, 0.263, 1.0],
    [0.024, 0.353, 0.710, 1.0],
    [0.459, 0.275, 0.396, 1.0],
    [1.000, 0.431, 0.349, 1.0],
    [1.000, 0.616, 0.506, 1.0],
];

const HIGHLIGHT_COLORS: &'static [[f32; 4]] = &[
    [0.214, 0.276, 0.45, 1.0],
    [0.619, 0.259, 0.445, 1.0],
    [0.0818, 0.654, 0.426, 1.0],
    [0.796, 0.458, 0.351, 1.0],
    [0.498, 0.487, 0.476, 1.0],
    [0.905, 0.905, 0.905, 1.0],
    [1.0, 1.0, 1.0, 1.0],
    [1.0, 0.125, 0.389, 1.0],
    [1.0, 0.684, 0.125, 1.0],
    [1.0, 0.936, 0.278, 1.0],
    [0.125, 1.0, 0.332, 1.0],
    [0.286, 0.726, 1.0, 1.0],
    [0.68, 0.65, 0.737, 1.0],
    [1.0, 0.592, 0.739, 1.0],
    [1.0, 0.875, 0.792, 1.0],
    [0.125, 0.125, 0.125, 1.0],
    [0.286, 0.196, 0.174, 1.0],
    [0.149, 0.21, 0.333, 1.0],
    [0.384, 0.239, 0.332, 1.0],
    [0.156, 0.447, 0.474, 1.0],
    [0.58, 0.301, 0.278, 1.0],
    [0.411, 0.339, 0.365, 1.0],
    [0.76, 0.698, 0.664, 1.0],
    [1.0, 0.988, 0.639, 1.0],
    [0.87, 0.192, 0.436, 1.0],
    [1.0, 0.508, 0.266, 1.0],
    [0.77, 1.0, 0.324, 1.0],
    [0.104, 0.835, 0.375, 1.0],
    [0.133, 0.469, 0.835, 1.0],
    [0.584, 0.423, 0.529, 1.0],
    [1.0, 0.54, 0.474, 1.0],
    [1.0, 0.713, 0.631, 1.0],
];

pub fn build_map(
    ui: &Ui,
    map_state: &mut MapState,
    screens: &ScreenMap,
    selected_partition: Option<&Partition>,
    partition_members: &FxHashMap<ScreenCoord, usize>,
    mut requested_center: Option<ScreenCoord>,
) -> Option<ScreenCoord> {
    let draw_list = ui.get_window_draw_list();
    let map_size = ui.content_region_avail();
    let [map_x_screen, map_y_screen] = ui.get_cursor_screen_pos();
    
    let line_thickness = get_line_thickness_for_zoom_level(map_state.zoom_level);
    let (cell_width, cell_height) = get_cell_size_for_zoom_level(map_state.zoom_level, map_state.aspect_ratio);
    
    // Recenter if map was resized
    if requested_center.is_none()
        && let Some(prev_geom) = &map_state.prev_geom
        && map_size != prev_geom.size
    {
        requested_center = Some(map_get_center_screen(prev_geom));
    }
    
    // Move the top left of the map to get the desired center
    if let Some(center) = requested_center {
        map_state.top_left = (
            center.0 - (map_size[0] / 2.0 / (cell_width + line_thickness)) as i32,
            center.1 - (map_size[1] / 2.0 / (cell_height + line_thickness)) as i32,
        );
    }
    
    // Pan
    if ui.is_mouse_clicked(MouseButton::Right) && ui.is_window_hovered() {
        map_state.is_dragging = true;
    }
    let pan =
        if map_state.is_dragging {
            ui.mouse_drag_delta(MouseButton::Right).into()
        }
        else {
            (0.0, 0.0)
        };
    
    let geom = calc_map_geometry(
        map_state,
        pan,
        ui.get_content_region_avail().into(),
    );
    map_state.prev_geom = Some(geom.clone());
    
    // When panning stops, we "commit" the current geometry
    if map_state.is_dragging && ui.is_mouse_released(MouseButton::Right) {
        map_state.is_dragging = false;
        map_state.top_left = (geom.x_min, geom.y_min);
        map_state.bias = (geom.origin_x, geom.origin_y);
    }
    
    let cols = (geom.x_max - geom.x_min + 1) as usize;
    let rows = (geom.y_max - geom.y_min + 1) as usize;
    let n_grid_cells = rows * cols;
    
    // Draw grid lines
    if get_line_thickness_for_zoom_level(map_state.zoom_level) > 0.0 {
        let mut x = map_x_screen + geom.origin_x;
        for _ in 0..cols {
            draw_list.add_line_v(x, map_y_screen, map_y_screen + map_size[1], [0.1, 0.1, 0.1], line_thickness);
            x += geom.cell_outer_width;
        }
        let mut y = map_y_screen + geom.origin_y;
        for _ in 0..rows {
            draw_list.add_line_h(map_x_screen, map_x_screen + map_size[0], y, [0.1, 0.1, 0.1], line_thickness);
            y += geom.cell_outer_height;
        }
    }
    
    // Helper functions
    let relative_to_screen_coords = |rel_pos: [f32; 2]| {
        [rel_pos[0] + map_x_screen, rel_pos[1] + map_y_screen]
    };
    let screen_to_relative_coords = |screen_pos: [f32; 2]| {
        [screen_pos[0] - map_x_screen, screen_pos[1] - map_y_screen]
    };
    let draw_rect_relative = |top_left_rel: [f32; 2], bottom_right_rel: [f32; 2], color: [f32; 4], filled: bool| {
        let top_left_screen = relative_to_screen_coords(top_left_rel);
        let bottom_right_screen = relative_to_screen_coords(bottom_right_rel);
        draw_list.add_rect(top_left_screen, bottom_right_screen, color)
            .filled(filled)
            .build();
    };
    let draw_screen_rect = |(x, y)| {
        let cell_pos = calc_cell_pos((x, y), &geom);
        let top_left = [
            cell_pos[0] + line_thickness,
            cell_pos[1] + line_thickness,
        ];
        let bottom_right = [
            top_left[0] + cell_width,
            top_left[1] + cell_height
        ];
        
        let partition_index = partition_members.get(&(x, y)).unwrap();
        let color_index = *partition_index % MAP_COLORS.len();
        
        let color = MAP_COLORS[color_index];
        draw_rect_relative(top_left, bottom_right, color, true);
        
        if cell_height >= 5.0 {
            let highlight_color = HIGHLIGHT_COLORS[color_index];
            draw_rect_relative(top_left, bottom_right, highlight_color, false);
        }
    };
    let draw_indicator = |(x, y), color| {
        let cell_pos = calc_cell_pos((x, y), &geom);
        let top_left = [
            cell_pos[0] + line_thickness,
            cell_pos[1] + line_thickness,
        ];
        let bottom_right = [
            top_left[0] + cell_width,
            top_left[1] + cell_height,
        ];
        draw_rect_relative(top_left, bottom_right, color, false);
    };
    
    // Now, we either iterate over screens (and check if they're on the map), or iterate over map cells
    // (and check if they contain a screen), whichever takes fewer iterations.
    if screens.len() <= n_grid_cells {
        for screen in screens.iter() {
            let (x, y) = screen.position;
            if x >= geom.x_min && x <= geom.x_max && y >= geom.y_min && y <= geom.y_max {
                draw_screen_rect((x, y));
            }
        }
    }
    else {
        for y in geom.y_min..=geom.y_max {
            for x in geom.x_min..=geom.x_max {
                if screens.index_of(&(x, y)).is_some() {
                    draw_screen_rect((x, y));
                }
            }
        }
    }
    
    // Draw partition outline
    if let Some(bounds) = selected_partition.map(|partition| partition.bounds())
        && !bounds.x.is_empty()
        && !bounds.y.is_empty()
    {
        let top_left = calc_cell_pos((bounds.x.start as i32, bounds.y.start as i32), &geom);
        let mut bottom_right = calc_cell_pos((bounds.x.end as i32, bounds.y.end as i32), &geom);
        bottom_right[0] += line_thickness;
        bottom_right[1] += line_thickness;
        draw_rect_relative(top_left, bottom_right, [1.0, 1.0, 1.0, 1.0], false);
    }
    
    // Hover and selection indicator
    if ui.is_window_hovered() {
        let mouse_pos = screen_to_relative_coords(ui.mouse_pos());
        let hovered_screen_pos = get_hovered_screen_pos(mouse_pos, &geom);
        
        if ui.is_mouse_clicked(MouseButton::Left) {
            if screens.index_of(&hovered_screen_pos).is_some()
                && map_state.selected_screen != Some(hovered_screen_pos)
            {
                map_state.selected_screen = Some(hovered_screen_pos);
            }
            else {
                map_state.selected_screen = None;
            }
        }
        
        if let Some(selected_pos) = &map_state.selected_screen {
            draw_indicator(*selected_pos, [1.0, 1.0, 0.0, 1.0]);
        }
        
        let hovered_cell_pos = calc_cell_pos(hovered_screen_pos, &geom);
        let hovered_top_left = [
            hovered_cell_pos[0] + line_thickness,
            hovered_cell_pos[1] + line_thickness,
        ];
        let hovered_bottom_right = [
            hovered_top_left[0] + cell_width,
            hovered_top_left[1] + cell_height,
        ];
        draw_rect_relative(hovered_top_left, hovered_bottom_right, [1.0, 1.0, 1.0, 1.0], false);
        
        // Zoom
        let wheel_delta = ui.get_mouse_wheel();
        if wheel_delta != 0.0 && !map_state.is_dragging {
            let new_zoom_level = (map_state.zoom_level + wheel_delta as i32).clamp(0, 12);
            let (new_cell_width, new_cell_height) = get_cell_size_for_zoom_level(new_zoom_level, map_state.aspect_ratio);
            let new_line_thickness = get_line_thickness_for_zoom_level(new_zoom_level);
            
            // The general idea here is to keep the point the mouse is hovering over in the same position as we zoom
            // We start by converting the pixel position to a proportion that is agnostic to the zoom level
            let mouse_pos_within_cell = [
                mouse_pos[0] - hovered_top_left[0],
                mouse_pos[1] - hovered_top_left[1]
            ];
            let mouse_pos_within_cell_proportion = [
                mouse_pos_within_cell[0] / geom.cell_outer_width,
                mouse_pos_within_cell[1] / geom.cell_outer_height,
            ];
            
            // Next, we use that proportion to calculate the pixel offset at the new zoom level
            let new_cell_outer_width = new_cell_width + new_line_thickness;
            let new_cell_outer_height = new_cell_height + new_line_thickness;
            let new_mouse_pos_within_cell = [
                f32::round(mouse_pos_within_cell_proportion[0] * new_cell_outer_width),
                f32::round(mouse_pos_within_cell_proportion[1] * new_cell_outer_height),
            ];
            
            // Working backwards, we can calculate where the top left of the hovered screen is at the new zoom level
            let new_hovered_cell_top_left = [
                mouse_pos[0] - new_mouse_pos_within_cell[0],
                mouse_pos[1] - new_mouse_pos_within_cell[1],
            ];
            
            // Since we know the map coordinates of the hovered screen and the pixel coordinates of its top left corner,
            // we can do some math to figure out the what screen appears in the top left of the map and where its
            // top left corner is.
            let blah_x = f32::ceil(new_hovered_cell_top_left[0] / new_cell_outer_width) as i32;
            let blah_y = f32::ceil(new_hovered_cell_top_left[1] / new_cell_outer_height) as i32;
            let new_top_left_screen = (
                hovered_screen_pos.0 - blah_x,
                hovered_screen_pos.1 - blah_y,
            );
            let new_bias = (
                new_hovered_cell_top_left[0] - blah_x as f32 * new_cell_outer_width,
                new_hovered_cell_top_left[1] - blah_y as f32 * new_cell_outer_height,
            );
            
            map_state.zoom_level = new_zoom_level;
            map_state.top_left = new_top_left_screen;
            map_state.bias = new_bias;
        }
        
        Some(hovered_screen_pos)
    }
    else if let Some(selected_pos) = &map_state.selected_screen {
        draw_indicator(*selected_pos, [1.0, 1.0, 0.0, 1.0]);
        None
    }
    else {
        None
    }
}

#[derive(Clone)]
pub struct MapGeometry {
    size: [f32; 2],
    x_min: i32,
    y_min: i32,
    x_max: i32,
    y_max: i32,
    origin_x: f32,
    origin_y: f32,
    cell_outer_width: f32,
    cell_outer_height: f32,
}

fn calc_map_geometry(map_state: &MapState, pan: (f32, f32), map_size: (f32, f32)) -> MapGeometry {
    let total_bias = (map_state.bias.0 + pan.0, map_state.bias.1 + pan.1);
    
    let (cell_width, cell_height) = get_cell_size_for_zoom_level(map_state.zoom_level, map_state.aspect_ratio);
    let line_thickness = get_line_thickness_for_zoom_level(map_state.zoom_level);
    let cell_outer_width = cell_width + line_thickness;
    let cell_outer_height = cell_height + line_thickness;
    
    let blah_x = f32::ceil(total_bias.0 / cell_outer_width) as i32;
    let blah_y = f32::ceil(total_bias.1 / cell_outer_height) as i32;
    
    let x_min = map_state.top_left.0 - blah_x;
    let y_min = map_state.top_left.1 - blah_y;
    
    let origin_x = total_bias.0 + (x_min - map_state.top_left.0) as f32 * cell_outer_width;
    let origin_y = total_bias.1 + (y_min - map_state.top_left.1) as f32 * cell_outer_height;
    
    let x_max = x_min + ((map_size.0 - origin_x) / cell_outer_width).floor() as i32;
    let y_max = y_min + ((map_size.1 - origin_y) / cell_outer_height).floor() as i32;
    
    MapGeometry {
        size: map_size.into(),
        x_min,
        y_min,
        x_max,
        y_max,
        cell_outer_width,
        cell_outer_height,
        origin_x,
        origin_y
    }
}

fn calc_cell_pos(screen_pos: ScreenCoord, geom: &MapGeometry) -> [f32; 2] {
    let dx = screen_pos.0 - geom.x_min;
    let dy = screen_pos.1 - geom.y_min;
    
    let x = geom.origin_x + dx as f32 * geom.cell_outer_width;
    let y = geom.origin_y + dy as f32 * geom.cell_outer_height;
    
    [x, y]
}

fn get_hovered_screen_pos(hover_pos: [f32; 2], geom: &MapGeometry) -> ScreenCoord {
    let offset_from_origin_x = hover_pos[0] - geom.origin_x;
    let offset_from_origin_y = hover_pos[1] - geom.origin_y;
    let screen_x = geom.x_min + (offset_from_origin_x / geom.cell_outer_width).floor() as i32;
    let screen_y = geom.y_min + (offset_from_origin_y / geom.cell_outer_height).floor() as i32;
    
    (screen_x, screen_y)
}

fn get_cell_size_for_zoom_level(zoom: i32, aspect_ratio: f32) -> (f32, f32) {
    let height = 1.6f32.powi(zoom).round();
    let width = (height * aspect_ratio).round().max(1.0);
    (width, height)
}

fn get_line_thickness_for_zoom_level(zoom: i32) -> f32 {
    if zoom < 4 { 0.0 } else { 1.0 }
}

pub fn map_get_center_screen(geom: &MapGeometry) -> ScreenCoord {
    let x = geom.x_min + (geom.size[0] / 2.0 / geom.cell_outer_width) as i32;
    let y = geom.y_min + (geom.size[1] / 2.0 / geom.cell_outer_height) as i32;
    (x, y)
}
