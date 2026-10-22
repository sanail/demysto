// Puts the main display at WIDTHxHEIGHT for as long as this runs: a mode
// configured for this application alone goes back when it quits.
ObjC.import("CoreGraphics");
ObjC.import("Foundation");

function run(argv) {
  var size = argv[0].split("x").map(Number), display = $.CGMainDisplayID();
  var modes = ObjC.castRefToObject($.CGDisplayCopyAllDisplayModes(display, null));
  for (var i = 0; i < modes.count; i++) {
    var mode = modes.objectAtIndex(i);
    // == rather than ===: the bridge hands the sizes over as objects.
    if ($.CGDisplayModeGetWidth(mode) == size[0] && $.CGDisplayModeGetHeight(mode) == size[1]) {
      var config = Ref();
      $.CGBeginDisplayConfiguration(config);
      $.CGConfigureDisplayWithDisplayMode(config[0], display, mode, null);
      $.CGCompleteDisplayConfiguration(config[0], $.kCGConfigureForAppOnly);
      console.log("switched");
      for (;;) $.NSThread.sleepForTimeInterval(60);
    }
  }
  throw new Error("no " + argv[0] + " mode");
}
