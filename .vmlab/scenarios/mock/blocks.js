// Shows the picture at argv[0] over the whole main display, stretched to it,
// until killed: osascript -l JavaScript blocks.js PATH TITLE
ObjC.import("Cocoa");

function run(argv) {
  const app = $.NSApplication.sharedApplication;
  app.setActivationPolicy($.NSApplicationActivationPolicyRegular);
  const frame = $.NSScreen.screens.objectAtIndex(0).frame; // the primary display
  const win = $.NSWindow.alloc.initWithContentRectStyleMaskBackingDefer(frame, $.NSWindowStyleMaskBorderless, $.NSBackingStoreBuffered, false);
  const view = $.NSImageView.alloc.initWithFrame(frame);
  view.image = $.NSImage.alloc.initWithContentsOfFile(argv[0]);
  view.imageScaling = $.NSImageScaleAxesIndependently;
  win.contentView = view;
  win.title = argv[1];
  win.makeKeyAndOrderFront(null);
  app.activateIgnoringOtherApps(true);
  app.run;
}
