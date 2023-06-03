use tui::{
  backend::Backend,
  layout::{Constraint, Rect},
  text::{Span, Spans, Text},
  widgets::{Block, Borders, Cell, LineGauge, Paragraph, Row, Table},
  Frame,
};

use super::{
  resource_tabs::draw_resource_tabs_block,
  utils::{
    get_gauge_style, horizontal_chunks, layout_block_default, loading, style_default,
    style_failure, style_highlight, style_logo, style_primary, style_secondary, table_header_style,
    vertical_chunks, vertical_chunks_with_margin,
  },
  HIGHLIGHT,
};
use crate::{
  app::{key_binding::DEFAULT_KEYBINDING, metrics::KubeNodeMetrics, ActiveBlock, App},
  banner::BANNER,
};

pub fn draw_overview<B: Backend>(f: &mut Frame<'_, B>, app: &mut App, area: Rect) {
  let mut constraints: Vec<Constraint> = vec![];
  if app.show_info_bar {
    constraints.push(Constraint::Length(9));
  }
  if app.show_filter {
    constraints.push(Constraint::Length(3));
  }
  constraints.push(Constraint::Min(10));

  let chunks = vertical_chunks(constraints, area);
  let mut chunks_index = 0;
  if app.show_info_bar {
    draw_status_block(f, app, chunks[chunks_index]);
    chunks_index += 1;
  }
  if app.show_filter {
    draw_filter(f, app, chunks[chunks_index]);
    chunks_index += 1;
  }
  draw_resource_tabs_block(f, app, chunks[chunks_index]);
}

pub fn draw_filter<B: Backend>(f: &mut Frame<'_, B>, app: &mut App, area: Rect) {
  let title = format!(
    " Filter {} (toggle: {}) ",
    DEFAULT_KEYBINDING.jump_to_filter.key, DEFAULT_KEYBINDING.toggle_filter.key
  );
  let mut block = layout_block_default(title.as_str());

  if app.get_current_route().active_block == ActiveBlock::Filter {
    block = block.style(style_secondary(app.light_theme));
    f.set_cursor(area.x + 2 + app.data.filter.len() as u16, area.y + 1);
  }

  let paragraph = Paragraph::new(app.data.filter.as_str()).block(block);

  f.render_widget(paragraph, area);
}

fn draw_status_block<B: Backend>(f: &mut Frame<'_, B>, app: &mut App, area: Rect) {
  let chunks = horizontal_chunks(
    vec![
      Constraint::Length(35),
      Constraint::Min(10),
      Constraint::Length(30),
      Constraint::Length(32),
    ],
    area,
  );

  draw_namespaces_block(f, app, chunks[0]);
  draw_context_info_block(f, app, chunks[1]);
  draw_cli_version_block(f, app, chunks[2]);
  draw_logo_block(f, app, chunks[3])
}

fn draw_logo_block<B: Backend>(f: &mut Frame<'_, B>, app: &mut App, area: Rect) {
  // Banner text with correct styling
  let text = format!(
    "{}\n v{} with ♥ in Rust {}",
    BANNER,
    env!("CARGO_PKG_VERSION"),
    nw_loading_indicator(app.is_loading)
  );
  let mut text = Text::from(text);
  text.patch_style(style_logo(app.light_theme));

  // Contains the banner
  let paragraph = Paragraph::new(text).block(Block::default().borders(Borders::ALL));
  f.render_widget(paragraph, area);
}

fn draw_cli_version_block<B: Backend>(f: &mut Frame<'_, B>, app: &mut App, area: Rect) {
  let block = layout_block_default(" CLI Info ");
  if !app.data.clis.is_empty() {
    let rows = app.data.clis.iter().map(|s| {
      let style = if s.status {
        style_primary(app.light_theme)
      } else {
        style_failure(app.light_theme)
      };
      Row::new(vec![
        Cell::from(s.name.as_ref()),
        Cell::from(s.version.as_ref()),
      ])
      .style(style)
    });

    let table = Table::new(rows)
      .block(block)
      .widths(&[Constraint::Percentage(50), Constraint::Percentage(50)]);
    f.render_widget(table, area);
  } else {
    loading(f, block, area, app.is_loading, app.light_theme);
  }
}

fn draw_context_info_block<B: Backend>(f: &mut Frame<'_, B>, app: &mut App, area: Rect) {
  let chunks = vertical_chunks_with_margin(
    vec![
      Constraint::Length(3),
      Constraint::Min(2),
      Constraint::Min(2),
    ],
    area,
    1,
  );

  let block = layout_block_default(" Context Info (toggle <i>) ");

  f.render_widget(block, area);

  let text = match &app.data.active_context {
    Some(active_context) => {
      vec![
        Spans::from(vec![
          Span::styled("Context: ", style_default(app.light_theme)),
          Span::styled(&active_context.name, style_primary(app.light_theme)),
        ]),
        Spans::from(vec![
          Span::styled("Cluster: ", style_default(app.light_theme)),
          Span::styled(&active_context.cluster, style_primary(app.light_theme)),
        ]),
        Spans::from(vec![
          Span::styled("User: ", style_default(app.light_theme)),
          Span::styled(&active_context.user, style_primary(app.light_theme)),
        ]),
      ]
    }
    None => {
      vec![Spans::from(Span::styled(
        "Context information not found",
        style_failure(app.light_theme),
      ))]
    }
  };

  let paragraph = Paragraph::new(text).block(Block::default());
  f.render_widget(paragraph, chunks[0]);

  let ratio = get_nm_ratio(app.data.node_metrics.as_ref(), |nm| nm.cpu_percent);
  let limited_ratio = if ratio > 1f64 { 1f64 } else { ratio };

  let cpu_gauge = LineGauge::default()
    .block(Block::default().title("CPU:"))
    .gauge_style(style_primary(app.light_theme))
    .line_set(get_gauge_style(app.enhanced_graphics))
    .ratio(limited_ratio)
    .label(Spans::from(format!("{:.0}%", ratio * 100.0)));
  f.render_widget(cpu_gauge, chunks[1]);

  let ratio = get_nm_ratio(app.data.node_metrics.as_ref(), |nm| nm.mem_percent);
  let limited_ratio = if ratio > 1f64 { 1f64 } else { ratio };

  let mem_gauge = LineGauge::default()
    .block(Block::default().title("Memory:"))
    .gauge_style(style_primary(app.light_theme))
    .line_set(get_gauge_style(app.enhanced_graphics))
    .ratio(limited_ratio)
    .label(Spans::from(format!("{:.0}%", ratio * 100.0)));
  f.render_widget(mem_gauge, chunks[2]);
}

fn draw_namespaces_block<B: Backend>(f: &mut Frame<'_, B>, app: &mut App, area: Rect) {
  let title = format!(
    " Namespaces {} (all: {}) ",
    DEFAULT_KEYBINDING.jump_to_namespace.key, DEFAULT_KEYBINDING.select_all_namespace.key
  );
  let mut block = layout_block_default(title.as_str());

  if app.get_current_route().active_block == ActiveBlock::Namespaces {
    block = block.style(style_secondary(app.light_theme))
  }

  if !app.data.namespaces.items.is_empty() {
    let rows = app.data.namespaces.items.iter().map(|s| {
      let style = if Some(s.name.clone()) == app.data.selected.ns {
        style_secondary(app.light_theme)
      } else {
        style_primary(app.light_theme)
      };
      Row::new(vec![
        Cell::from(s.name.as_ref()),
        Cell::from(s.status.as_ref()),
      ])
      .style(style)
    });

    let table = Table::new(rows)
      .header(table_header_style(vec!["Name", "Status"], app.light_theme))
      .block(block)
      .highlight_style(style_highlight())
      .highlight_symbol(HIGHLIGHT)
      .widths(&[Constraint::Length(22), Constraint::Length(6)]);

    f.render_stateful_widget(table, area, &mut app.data.namespaces.state);
  } else {
    loading(f, block, area, app.is_loading, app.light_theme);
  }
}

// Utility methods

/// covert percent value from metrics to ratio that gauge can understand
fn get_nm_ratio(node_metrics: &[KubeNodeMetrics], f: fn(b: &KubeNodeMetrics) -> f64) -> f64 {
  if !node_metrics.is_empty() {
    let sum = node_metrics.iter().map(f).sum::<f64>();
    (sum / node_metrics.len() as f64) / 100f64
  } else {
    0f64
  }
}

fn nw_loading_indicator<'a>(loading: bool) -> &'a str {
  if loading {
    "..."
  } else {
    ""
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  #[allow(clippy::float_cmp)]
  fn test_get_nm_ratio() {
    let mut app = App::default();
    assert_eq!(
      get_nm_ratio(app.data.node_metrics.as_ref(), |nm| nm.cpu_percent),
      0.0f64
    );
    app.data.node_metrics = vec![
      KubeNodeMetrics {
        cpu_percent: 80f64,
        ..KubeNodeMetrics::default()
      },
      KubeNodeMetrics {
        cpu_percent: 60f64,
        ..KubeNodeMetrics::default()
      },
    ];
    assert_eq!(
      get_nm_ratio(app.data.node_metrics.as_ref(), |nm| nm.cpu_percent),
      0.7f64
    );
  }
}
