#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <fcntl.h>
#include <unistd.h>
#include <sys/ioctl.h>
#include <sys/mman.h>
#include <signal.h>
#include <pthread.h>
#include <errno.h>
#include <getopt.h>
#include "hvisor.h"
#include "virtio.h"
#include "log.h"
#include "event_monitor.h"
#include "cJSON.h"
#include "zone_config.h"

static void __attribute__((noreturn)) help(int exit_status) {
    printf("Invalid Parameters!\n");
    exit(exit_status);
}

static void* read_file(char* filename, u64* filesize) {
    int fd;
    struct stat st;
    void *buf;
    ssize_t len;
    fd = open(filename, O_RDONLY);

    if(fd < 0) {
        perror("read_file: open file failed");
        exit(1);
    }

    if (fstat(fd, &st) < 0) {
        perror("read_file: fstat failed");
        exit(1);
    }
	long page_size = sysconf(_SC_PAGESIZE);
	ssize_t buf_size = (st.st_size + page_size - 1) & ~(page_size - 1);
    buf = malloc(buf_size);
    memset(buf, 0, buf_size);
	len = read(fd, buf, st.st_size);

    if (len < 0) {
        perror("read_file: read failed");
        exit(1);
    }
    if (filesize)
        *filesize = len;
    close(fd);
    return buf;
}

int open_dev() {
    int fd = open("/dev/hvisor", O_RDWR);
    if (fd < 0) {
        perror("open hvisor failed");
        exit(1);
    }
    return fd;
}

static u64 load_image_to_memory(const char *path, u64 load_paddr) {
    u64 size, page_size, map_size;
    int fd;
    void *image_content, *virt_addr;

    fd = open_dev();
    // Load image content into memory
    image_content = read_file(path, &size);

    page_size = sysconf(_SC_PAGESIZE);
    map_size = (size + page_size - 1) & ~(page_size - 1);

    // Map the physical memory to virtual memory
    virt_addr = (u64)mmap(NULL, map_size, PROT_READ | PROT_WRITE, MAP_SHARED, fd, load_paddr);

    if (virt_addr == MAP_FAILED) {
        perror("Error mapping memory");
        exit(1);
    }

    memmove(virt_addr, image_content, map_size);

    free(image_content);
    munmap(virt_addr, map_size);

    close(fd);
    return map_size;
}

static int zone_start_from_json(const char *json_config_path, zone_config_t *config) {
    FILE *file = fopen(json_config_path, "r");
    if (file == NULL) {
        perror("Error opening file");
        exit(1);
    }
    fseek(file, 0, SEEK_END);
    long file_size = ftell(file);
    fseek(file, 0, SEEK_SET);
    char *buffer = malloc(file_size + 1);
    fread(buffer, 1, file_size, file);
    fclose(file);
    buffer[file_size] = '\0';

    // parse JSON
    cJSON *root = cJSON_Parse(buffer);
    cJSON *zone_id_json = cJSON_GetObjectItem(root, "zone_id");
    cJSON *cpus_json = cJSON_GetObjectItem(root, "cpus");
    cJSON *memory_regions_json = cJSON_GetObjectItem(root, "memory_regions");
    cJSON *kernel_filepath_json = cJSON_GetObjectItem(root, "kernel_filepath");
    cJSON *dtb_filepath_json = cJSON_GetObjectItem(root, "dtb_filepath");
    cJSON *kernel_load_paddr_json = cJSON_GetObjectItem(root, "kernel_load_paddr");
    cJSON *dtb_load_paddr_json = cJSON_GetObjectItem(root, "dtb_load_paddr");
    cJSON *entry_point_json = cJSON_GetObjectItem(root, "entry_point");
    cJSON *kernel_args_json = cJSON_GetObjectItem(root, "kernel_args");
    cJSON *interrupts_json = cJSON_GetObjectItem(root, "interrupts");

    config->zone_id = zone_id_json->valueint;

    int num_cpus = cJSON_GetArraySize(cpus_json);

    for (int i = 0; i < num_cpus; i++) {
        config->cpus |= (1 << cJSON_GetArrayItem(cpus_json, i)->valueint);
    }

    int num_memory_regions = cJSON_GetArraySize(memory_regions_json);
    int num_interrupts = cJSON_GetArraySize(interrupts_json);

    if (num_memory_regions > CONFIG_MAX_MEMORY_REGIONS || num_interrupts > CONFIG_MAX_INTERRUPTS) {
        fprintf(stderr, "Exceeded maximum allowed regions/interrupts.\n");
        cJSON_Delete(root);
        fclose(file);
        free(buffer);
        return -1;
    }

    config->num_memory_regions = num_memory_regions;
    for (int i = 0; i < num_memory_regions; i++) {
        cJSON *region = cJSON_GetArrayItem(memory_regions_json, i);
        memory_region_t *mem_region = &config->memory_regions[i];

        const char *type_str = cJSON_GetObjectItem(region, "type")->valuestring;
        if (strcmp(type_str, "ram") == 0) {
            mem_region->type = MEM_TYPE_RAM;
        } else if (strcmp(type_str, "io") == 0) {
            mem_region->type = MEM_TYPE_IO;
        } else if (strcmp(type_str, "virtio") == 0) {
            mem_region->type = MEM_TYPE_VIRTIO;
        } else {
            printf("Unknown memory region type: %s\n", type_str);
            mem_region->type = -1; // invalid type
        }

        mem_region->physical_start = strtoull(cJSON_GetObjectItem(region, "physical_start")->valuestring, NULL, 16);
        mem_region->virtual_start = strtoull(cJSON_GetObjectItem(region, "virtual_start")->valuestring, NULL, 16);
        mem_region->size = strtoull(cJSON_GetObjectItem(region, "size")->valuestring, NULL, 16);

        printf("memory_region %d: type %d, physical_start %llx, virtual_start %llx, size %llx\n",
            i, mem_region->type, mem_region->physical_start, mem_region->virtual_start, mem_region->size);
    }

    config->num_interrupts = num_interrupts;
    for (int i = 0; i < num_interrupts; i++) {
        config->interrupts[i] = cJSON_GetArrayItem(interrupts_json, i)->valueint;
    }

    config->entry_point = strtoull(entry_point_json->valuestring, NULL, 16);

    config->kernel_load_paddr = strtoull(kernel_load_paddr_json->valuestring, NULL, 16);

    config->dtb_load_paddr = strtoull(dtb_load_paddr_json->valuestring, NULL, 16);

    // Load kernel image to memory
    config->kernel_size = load_image_to_memory(kernel_filepath_json->valuestring, strtoull(kernel_load_paddr_json->valuestring, NULL, 16));

    // Load dtb to memory
    config->dtb_size = load_image_to_memory(dtb_filepath_json->valuestring, strtoull(dtb_load_paddr_json->valuestring, NULL, 16));

    // strncpy(config->kernel_args, kernel_args_json->valuestring, CONFIG_KERNEL_ARGS_MAXLEN);

    cJSON_Delete(root);  // delete cJSON object
    free(buffer);

    int fd = open_dev();
    int err = ioctl(fd, HVISOR_ZONE_START, config);
    if (err)
        perror("zone_start: ioctl failed");
    close(fd);

    return 0;
}

// ./hvisor zone start <path_to_config_file>
static int zone_start(int argc, char *argv[]) {
    int fd, err, opt, zone_id;
	char *image_path = NULL, *dtb_filepath = NULL, *json_config_path = NULL;
    zone_config_t config;
	u64 image_address, dtb_address;

	if (argc != 4) {
		help(1);
	}
	json_config_path = argv[3];

    memset(&config, 0, sizeof(zone_config_t));
	zone_start_from_json(json_config_path, &config);

    return 0;
}

// ./hvisor zone shutdown -id 1
static int zone_shutdown(int argc, char *argv[]) {
	if (argc != 2 || strcmp(argv[0], "-id") != 0) {
        help(1);
	}
	__u64 zone_id;
	sscanf(argv[1], "%llu", &zone_id);
	int fd = open_dev();
	int err = ioctl(fd, HVISOR_ZONE_SHUTDOWN, zone_id);
	if (err)
		perror("zone_shutdown: ioctl failed");
	close(fd);
	return err;
}

int main(int argc, char *argv[])
{
    int err;

    if (argc < 2)
        help(1);

    if (strcmp(argv[1], "zone") == 0 && strcmp(argv[2], "start") == 0) {
        err = zone_start(argc, argv);
    } else if (strcmp(argv[1], "zone") == 0 && strcmp(argv[2], "shutdown") == 0){
		err = zone_shutdown(argc - 3, &argv[3]);
	}else if (strcmp(argv[1], "virtio") == 0 && strcmp(argv[2], "start") == 0) {
        err = virtio_start(argc, argv);
    } else {
        help(1);
    }

    return err ? 1 : 0;
}



