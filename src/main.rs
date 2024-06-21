mod editor;
use editor::Editor;
fn main() {

    let results = Editor::read();
    let mut editor = Editor::default(results);
    editor.run();


}