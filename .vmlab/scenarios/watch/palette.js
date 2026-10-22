// Prints the bounds of Demysto's windows on screen each time they change, or
// "none" when there are none: fast enough to see a resize no user should.
ObjC.import("CoreGraphics");

// The Palette is a panel at the floating level; the tray icon and every other
// window of Demysto's sit at other levels.
var FLOATING = 3;

function run() {
  var last = null;
  for (;;) {
    var windows = ObjC.deepUnwrap(ObjC.castRefToObject(
      $.CGWindowListCopyWindowInfo($.kCGWindowListOptionOnScreenOnly, 0))) || [];
    var now = windows
      .filter(function (w) { return w.kCGWindowOwnerName === "Demysto" && w.kCGWindowLayer === FLOATING; })
      .map(function (w) { var b = w.kCGWindowBounds; return [b.X, b.Y, b.Width, b.Height].join(" "); })
      .join(" | ") || "none";
    if (now !== last) {
      console.log(now);
      last = now;
    }
  }
}
