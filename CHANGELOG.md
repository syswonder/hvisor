# Changelog

> ⚠️ Please update this file for any changes to the hvisor project along with your name and GitHub profile link under the CURRENT section.

## CURRENT - v0.1.2

- [feature] riscv64: add vplic struct ([Jingyu Liu](https://github.com/liulog))
- [feature] riscv64: add aclint support ([Jingyu Liu](https://github.com/liulog))

## History Release

### hvisor v0.1.1

- [platform] seperate board definitions into `platform` folder with re-designed cargo feature system for hvisor ([wheatfox](https://github.com/enkerewpo))

### hvisor v0.1.0

- [platform] architecture officially supported: riscv64, loongarch64 ([Jingyu Liu](https://github.com/liulog), [wheatfox](https://github.com/enkerewpo))
- [tool] adapting hvisor-tool virtio-gpu, virtio-console ([KouweiLee](https://github.com/KouweiLee), [Roxy](https://github.com/Misaka19986), [wheatfox](https://github.com/enkerewpo))
- [bugfix] refactor aarch64 pagetable code ([dallasxy](https://github.com/dallasxy))
- [platform] Xilinx Ultrascale+ ZCU102 PS processor support ([Ren HangQi](https://github.com/ForeverYolo))
- [platform] Loongson 3A5000+7A2000 support ([wheatfox](https://github.com/enkerewpo), [BoneInscri](https://github.com/BoneInscri))
- [feature] SMMUv3 support ([Zhongkai Xu](https://github.com/ZhongkaiXu))
- [feature] PCIe support ([Zhongkai Xu](https://github.com/ZhongkaiXu), [dallasxy](https://github.com/dallasxy), [Ren HangQi](https://github.com/ForeverYolo))
- [feature] network interface card support ([Ren HangQi](https://github.com/ForeverYolo))
- [feature] riscv64: IOMMU support ([Jingyu Liu](https://github.com/liulog))
- [feature] aarch64: GICv2 support ([Ren HangQi](https://github.com/ForeverYolo))
- [feature] basic inter-vm communication(ivc) support ([KouweiLee](https://github.com/KouweiLee))
- [test] unittest and github ci support ([wheatfox](https://github.com/enkerewpo))
- [tool] hvisor-tool: support virtio-console, virtio-blk, virtio-net ([KouweiLee](https://github.com/KouweiLee))
- [platform] basic support for riscv64 ([likey99](https://github.com/likey99))
- [tool] aarch64: management tool in root zone linux, can create, stop, suspend and destroy working zones ([KouweiLee](https://github.com/KouweiLee))
- [platform] basic support for aarch64 with root and nonroot zone booting ([Nehckl](https://github.com/Inquisitor-201))
