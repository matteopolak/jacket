// test: only(.MACOS) skip_on(.WASM) serial
// AppKit facade: cursor class lookups, panel construction and the file URL pasteboard type.
#import "Basic";
#import "Objective_C";
#import "Objective_C/AppKit";

main :: () {
    #if OS == .MACOS {
        // Without a window server the class getters may return nil; they must still be callable.
        NSCursor.arrowCursor();
        NSCursor.IBeamCursor();
        NSCursor.pointingHandCursor();
        NSCursor.resizeLeftRightCursor();

        save := NSSavePanel.savePanel();

        assert(save != null);
        NSWindow.setTitle(save, "Save");
        NSSavePanel.setNameFieldStringValue(save, "out.txt");

        open := NSOpenPanel.openPanel();

        assert(open != null);
        NSOpenPanel.setCanChooseFiles(open, YES);

        assert(NSPasteboardTypeFileURL != null);
        assert(NSModalResponseOK == 1);
    }
}
