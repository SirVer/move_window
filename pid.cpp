#include <Carbon/Carbon.h>
#include <ApplicationServices/ApplicationServices.h>

#include <stdint.h>
#include <vector>

using namespace std;

extern "C" {

void _frontmost_process(char *buffer, int n) {
	ProcessSerialNumber psn = { 0L, 0L };
	OSStatus err = GetFrontProcess(&psn);

	CFStringRef processName = NULL;
	err = CopyProcessName(&psn, &processName);

	CFStringGetCString(processName, buffer, n, 0);

	CFRelease(processName);
}

// Code below inspired by
// http://hints.macworld.com/article.php?story=20090413120929454
void _get_display_resolution(CGDirectDisplayID *dispArray,  CGDisplayCount dispNum, int32_t *w, int32_t *h) {
	CFDictionaryRef currentMode = CGDisplayCurrentMode (dispArray[dispNum]);
	CFNumberRef number = (const CFNumberRef) CFDictionaryGetValue (currentMode, kCGDisplayWidth);
	CFNumberGetValue (number, kCFNumberSInt32Type, w);
	number = (const CFNumberRef) CFDictionaryGetValue (currentMode, kCGDisplayHeight);
	CFNumberGetValue (number, kCFNumberSInt32Type, h);
}

void _display_resolutions(vector<vector<int32_t> > * rv) {
	CGDisplayCount maxDisplays = 10;
	CGDirectDisplayID displays[maxDisplays];
	CGDisplayCount dispCount;

	CGGetOnlineDisplayList(maxDisplays, displays, &dispCount);

	int32_t	w, h;
	for (int i = 0 ; i < dispCount ;  i++ ) {
		vector<int32_t> cur_res;
		_get_display_resolution(displays, i, &w, &h);
		cur_res.push_back(w);
		cur_res.push_back(h);
		rv->push_back(cur_res);
	}
}

}


