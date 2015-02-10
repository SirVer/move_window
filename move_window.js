#!osascript -lJavaScript 

ObjC.import('AppKit')

// Returns the main screen boundaries.
function getResolution() {
   var frame = $.NSScreen.mainScreen.frame.size;
   return { width: frame.width, height: frame.height };
}

// Returns the name of the frontMost Applicatine, i.e. 'iTerm' or 'Finder'.
function getFrontmostApplicationName() {
   return $.NSWorkspace
      .sharedWorkspace
      .activeApplication
      .objectForKey('NSApplicationName')
      .UTF8String;
}

function parseArgs(args) {
   res = getResolution();

   // TODO(sirver): support more than one screen
   var screen = 0; 
   args = args.substr(1);

   var xratio = 1;
   var xpos_s = 0;
   var yratio = 1;
   var ypos_s = 0;

   if (args) { xratio = parseInt(args[0]); args = args.substr(1); }
   if (args) { xpos_s = parseInt(args[0]); args = args.substr(1); }
   var xpos_e = xpos_s;
   if (args && args[0] === '-') {
      xpos_e = parseInt(args[1]); args = args.substr(2);
   }
   if (args) { yratio = parseInt(args[0]); args = args.substr(1); }
   if (args) { ypos_s = parseInt(args[0]); args = args.substr(1); }
   var ypos_e = ypos_s;
   if (args && args[1] === '-') {
      ypos_e = parseInt(args[0]); args = args.substr(2);
   }

   var x, y; 
   if (screen == 0) {
      // Take Menu bar into account.
      x = 0;
      y = 22;
      res.height -= 22;
   }

    var oneWidth = (res.width / xratio);
    var oneHeight = (res.height / yratio);
    console.log(oneWidth, oneHeight, xpos_s, xpos_e);

    return {
       x: x + oneWidth * xpos_s,
       y: y + oneHeight * ypos_s,
       width: oneWidth * (xpos_e - xpos_s + 1),
       height: oneHeight * (ypos_e - ypos_s + 1),
    }
}

function run(args) {
   var app = getFrontmostApplicationName();
   var win = Application(app).windows[0];
   var newBounds = parseArgs(args[0]);
   console.log("newBounds:", JSON.stringify(newBounds));
   if (newBounds.width > 5 && newBounds.height > 5) {
      win.bounds = newBounds;
   }
}

