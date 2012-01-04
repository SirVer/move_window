#include <Carbon/Carbon.h>

void get_frontmost_process(char *buffer, int n) {
	ProcessSerialNumber psn = { 0L, 0L };
	OSStatus err = GetFrontProcess(&psn);

	CFStringRef processName = NULL;
	err = CopyProcessName(&psn, &processName);

	CFStringGetCString(processName, buffer, n, 0);

	CFRelease(processName);
}


