/* SPDX-License-Identifier: MulanPSL-2.0 */
#include "test_common.h"
#include <dirent.h>
#include <fcntl.h>
#include <sys/stat.h>

int main(void)
{
    const char *name = "t_dir_iterate";
    const char *dir = "/tmp/abi_dir_iterate";
    if (mkdir(dir, 0755) != 0 && errno != EEXIST)
        return abi_errno_fail(name, "mkdir");
    char file[128];
    snprintf(file, sizeof(file), "%s/child", dir);
    int fd = open(file, O_CREAT | O_WRONLY | O_TRUNC, 0644);
    if (fd < 0)
        return abi_errno_fail(name, "open child");
    close(fd);
    DIR *d = opendir(dir);
    if (!d)
        return abi_errno_fail(name, "opendir");
    int found = 0;
    struct dirent *de;
    while ((de = readdir(d)) != NULL) {
        if (strcmp(de->d_name, "child") == 0)
            found = 1;
    }
    closedir(d);
    unlink(file);
    rmdir(dir);
    if (!found)
        return abi_fail(name, "child not found");
    return abi_pass(name);
}
