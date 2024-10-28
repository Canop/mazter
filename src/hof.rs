use {
    crate::*,
    termimad::{
        MadSkin,
        minimad::{
            Alignment,
            Col,
            OwningTemplateExpander,
            TableBuilder,
        },
    },
};

// display the hall of fame
pub fn print() -> anyhow::Result<()> {
    let hof = Database::hof()?;
    if hof.is_empty() {
        println!("The Hall of Fame is empty");
        return Ok(());
    }
    let mut expander = OwningTemplateExpander::new();
    for entry in &hof {
        expander
            .sub("rows")
            .set("user", &entry.user)
            .set("level", entry.level);
    }
    let mut tbl = TableBuilder::default();
    tbl.col(Col::new("**User**", "${user}"));
    tbl.col(Col::new("**Level**", "${level}").align_content(Alignment::Right));
    let skin = MadSkin::default();
    skin.print_owning_expander_md(&expander, &tbl);
    Ok(())
}
