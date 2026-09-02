/* linuxdeploy's AppImage output plugin, with the host's own libraries dropped
 * from the AppDir on the way past. See ADR-0016.
 *
 * The plugin is the last thing to touch the AppDir before it is packed, which
 * is the only place this can be done. linuxdeploy has `--exclude-library`, but
 * the Tauri bundler passes it no arguments of ours, and the GTK plugin — which
 * runs after the exclusion would have applied, and calls linuxdeploy again
 * itself — puts the library back regardless.
 *
 * A compiled program rather than the shell script this obviously wants to be:
 * before running a tool the bundler zeroes three bytes at offset 8, the
 * AppImage magic, so that the tool runs without FUSE. In an ELF those bytes are
 * padding and nothing reads them; in a script they are the middle of the
 * shebang line, and the shim becomes a file the kernel cannot start.
 */
#include <glob.h>
#include <stdio.h>
#include <string.h>
#include <unistd.h>

#ifndef PLUGIN
#error "compile with -DPLUGIN=\"<path to the real output plugin>\""
#endif

/* What the AppImage excludelist says an AppImage must leave to the host, and
 * that linuxdeploy's own copy of that list — taken before the entry was
 * added — does not name. */
static const char *const excluded[] = {"libwayland-client.so*"};

static int drop(const char *appdir, const char *pattern) {
  static const char *const dirs[] = {"usr/lib*", "usr/lib*/*"};
  int failed = 0;

  for (size_t d = 0; d < sizeof dirs / sizeof *dirs; d++) {
    char where[4096];
    glob_t found;

    snprintf(where, sizeof where, "%s/%s/%s", appdir, dirs[d], pattern);
    if (glob(where, 0, NULL, &found) == 0) {
      for (size_t i = 0; i < found.gl_pathc; i++) {
        if (unlink(found.gl_pathv[i]) == 0) {
          fprintf(stderr, "left to the host: %s\n", found.gl_pathv[i]);
        } else {
          perror(found.gl_pathv[i]);
          failed = 1;
        }
      }
    }
    globfree(&found);
  }

  return failed;
}

int main(int argc, char **argv) {
  const char *appdir = NULL;

  for (int i = 1; i < argc; i++) {
    if (strcmp(argv[i], "--appdir") == 0 && i + 1 < argc) appdir = argv[++i];
    else if (strncmp(argv[i], "--appdir=", 9) == 0) appdir = argv[i] + 9;
  }

  /* linuxdeploy also calls a plugin to ask what it is (`--plugin-api-version`,
   * `--plugin-type`). Those calls name no AppDir and want a plain answer. */
  if (appdir) {
    for (size_t i = 0; i < sizeof excluded / sizeof *excluded; i++) {
      if (drop(appdir, excluded[i]) != 0) return 1;
    }
  }

  argv[0] = (char *)PLUGIN;
  execv(PLUGIN, argv);
  perror(PLUGIN);
  return 127;
}
