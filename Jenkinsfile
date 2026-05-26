/** ARCH/BOARD come from matrix BID (arch/board); ci.yaml must not duplicate them. */
def parseBid(String bid) {
    def parts = (bid ?: '').split('/', 2)
    if (parts.size() != 2 || !parts[0] || !parts[1]) {
        error("invalid BID: ${bid}")
    }
    return [arch: parts[0], board: parts[1]]
}

def parseCiBuildArgs(cfg) {
    def buildArgs = [:]
    if (!cfg?.build_args) {
        return buildArgs
    }
    cfg.build_args.each { item ->
        if (item instanceof Map) {
            item.each { k, v ->
                buildArgs[k.toString()] = v.toString()
            }
        } else {
            def parts = item.toString().split('=', 2)
            if (parts.size() == 2) {
                buildArgs[parts[0]] = parts[1]
            }
        }
    }
    return buildArgs
}

def matrixCellDir() {
    def bid = env.BID ?: ''
    return "${env.WORKSPACE}/.matrix/${bid.replace('/', '__')}"
}

def loadCiYaml() {
    def data = readYaml file: 'jenkins/ci.yaml'
    def bids = data?.bids
    if (!(bids instanceof List)) {
        error("jenkins/ci.yaml: 'bids' must be a list")
    }
    return data
}

def getBidConfig(ci, String bid) {
    return (ci.bids ?: []).find { entry ->
        return (entry?.bid ?: '').toString().trim() == bid
    }
}

def normalizeToolArch(String arch) {
    def raw = (arch ?: '').toString().trim()
    def mapping = [
        'aarch64'    : 'arm64',
        'arm64'      : 'arm64',
        'riscv64'    : 'riscv',
        'riscv'      : 'riscv',
        'loongarch64': 'loongarch',
        'loongarch'  : 'loongarch',
    ]
    return mapping.get(raw, raw)
}

/** GitHub Check name for this matrix cell */
def matrixCheckName() {
    return (env.BID ?: '').toString()
}

def publishGithubCheckInProgress(String checkName) {
    publishChecks(
        name: checkName,
        title: checkName,
        summary: 'In progress',
        status: 'IN_PROGRESS',
        conclusion: 'NONE',
        detailsURL: "${env.RUN_DISPLAY_URL ?: env.BUILD_URL}",
    )
}

def publishGithubCheckCompleted(String checkName, String conclusion) {
    publishChecks(
        name: checkName,
        title: checkName,
        summary: conclusion == 'SUCCESS' ? 'Passed' : 'Failed',
        status: 'COMPLETED',
        conclusion: conclusion,
        detailsURL: "${env.RUN_DISPLAY_URL ?: env.BUILD_URL}",
    )
}

def publishMatrixCheckInProgress() {
    publishGithubCheckInProgress(matrixCheckName())
}

def publishMatrixCheckCompleted(String conclusion) {
    publishGithubCheckCompleted(matrixCheckName(), conclusion)
}

/** Kconfig: venv + tools/kconfig/kconfig_cli.py (via make defconfig). Keep in sync with Makefile. */
def kconfigSetupShell(String arch, String board) {
    return """
        export PATH=${env.CARGO_HOME}/bin:${env.TOOLCHAIN_PATHS}:\$PATH
        chmod +x tools/kconfig/bootstrap_venv.sh tools/kconfig/host_config.sh tools/kconfig/save_defconfig.sh 2>/dev/null || true
        if [ ! -x tools/kconfig/.venv/bin/python ]; then
            ./tools/kconfig/bootstrap_venv.sh
        fi
        make defconfig ARCH=${arch} BOARD=${board}
    """
}

pipeline {
    agent any

    options {
        timestamps()
    }

    post {
        always {
            echo "=== DEBUG: Branch ${env.BRANCH_NAME} ==="
            echo "=== DEBUG: Commit ${env.GIT_COMMIT} ==="
        }
    }

    environment {
        HVISOR_TOOL_URL = 'https://github.com/syswonder/hvisor-tool.git'
        HVISOR_TOOL_PATH = 'hvisor-tool'
        RUST_HOME = '/usr/local/rustup'
        CARGO_HOME = '/usr/local/cargo'
        QEMU_PATH = '/home/light/DEMO/qemu-9.2.3/build'
        TEST_IMG_BASE = '/home/light/DEMO/syswonder/test_img'
        RISCV_TOOLCHAIN_PATH = '/home/light/DEMO/toolchain/riscv64-glibc-ubuntu-24.04-gcc'
        AARCH64_TOOLCHAIN_PATH = '/home/light/DEMO/toolchain/gcc-arm-10.3-2021.07-x86_64-aarch64-none-linux-gnu'
        LOONGARCH64_TOOLCHAIN_PATH = '/home/light/DEMO/toolchain/loongarch_cross_tools'
        // All toolchain bins on PATH; same for every matrix cell (no per-arch selection).
        TOOLCHAIN_PATHS = "${env.RISCV_TOOLCHAIN_PATH}/bin:${env.AARCH64_TOOLCHAIN_PATH}/bin:${env.LOONGARCH64_TOOLCHAIN_PATH}/bin"
    }

    stages {
        stage('Checkout') {
            steps {
                // Ensure no stale files from previous builds.
                deleteDir()
                checkout scm
            }
        }

        stage('Linter') {
            steps {
                script {
                    publishGithubCheckInProgress('linter')
                    sh """
                        export PATH=${env.CARGO_HOME}/bin:${env.TOOLCHAIN_PATHS}:\$PATH
                        make fmt-test
                    """
                }
            }
            post {
                success {
                    script { publishGithubCheckCompleted('linter', 'SUCCESS') }
                }
                failure {
                    script { publishGithubCheckCompleted('linter', 'FAILURE') }
                }
                unstable {
                    script { publishGithubCheckCompleted('linter', 'FAILURE') }
                }
            }
        }

        stage('License checker') {
            steps {
                script {
                    publishGithubCheckInProgress('license-checker')
                    sh """
                        chmod +x tools/license_checker.sh
                        ./tools/license_checker.sh
                    """
                }
            }
            post {
                success {
                    script { publishGithubCheckCompleted('license-checker', 'SUCCESS') }
                }
                failure {
                    script { publishGithubCheckCompleted('license-checker', 'FAILURE') }
                }
                unstable {
                    script { publishGithubCheckCompleted('license-checker', 'FAILURE') }
                }
            }
        }

        stage('Multi-Platform Matrix') {
            matrix {
                axes {
                    axis {
                        name 'BID'
                        values(
                            'aarch64/imx8mp',
                            'aarch64/ok6254-c',
                            'aarch64/phytium-pi',
                            'aarch64/qemu-gicv2',
                            'aarch64/qemu-gicv3',
                            'aarch64/rk3568',
                            'aarch64/rk3588',
                            'aarch64/sysoul_x3300',
                            'aarch64/zcu102',
                            'loongarch64/ls3a5000',
                            'loongarch64/ls3a6000',
                            'riscv64/hifive-premier-p550',
                            'riscv64/megrez',
                            'riscv64/qemu-aia',
                            'riscv64/qemu-plic',
                            'riscv64/ur-dp1000',
                            'x86_64/ecx-2300f-peg',
                            'x86_64/nuc14mnk',
                            'x86_64/qemu',
                        )
                    }
                }

                stages {
                    stage('Prepare cell workspace') {
                        steps {
                            script {
                                publishMatrixCheckInProgress()
                                def cellWs = matrixCellDir()
                                sh """
                                    mkdir -p '${cellWs}'
                                    rsync -a --delete \\
                                        --exclude '.matrix/' \\
                                        --exclude '.jenkins-matrix/' \\
                                        '${env.WORKSPACE}/' '${cellWs}/'
                                """
                            }
                        }
                    }

                    stage('Compile') {
                        steps {
                            dir(matrixCellDir()) {
                                script {
                                    def bid = parseBid(env.BID)
                                    def arch = bid.arch
                                    def board = bid.board
                                    echo "Compile hvisor [BID=${env.BID}, ARCH=${arch}, BOARD=${board}]"
                                    sh kconfigSetupShell(arch, board)
                                    if (arch != 'x86_64') {
                                        sh """
                                            export PATH=${env.CARGO_HOME}/bin:${env.TOOLCHAIN_PATHS}:\$PATH
                                            make dtb ARCH=${arch} BOARD=${board}
                                        """
                                    }
                                    sh """
                                        export PATH=${env.CARGO_HOME}/bin:${env.TOOLCHAIN_PATHS}:\$PATH
                                        make all ARCH=${arch} BOARD=${board} MODE=release
                                    """
                                }
                            }
                        }
                    }

                    stage('Build hvisor-tool') {
                        when {
                            expression {
                                return getBidConfig(loadCiYaml(), env.BID) != null
                            }
                        }
                        steps {
                            dir(matrixCellDir()) {
                                script {
                                    def ci = loadCiYaml()
                                    def bidCfg = getBidConfig(ci, env.BID)
                                    def buildArgs = parseCiBuildArgs(bidCfg)
                                    def bidTool = parseBid(env.BID)
                                    def tarch = normalizeToolArch(buildArgs.TARCH ?: bidTool.arch)
                                    def kdir = buildArgs.KDIR
                                    if (!kdir) {
                                        error("jenkins/ci.yaml BID=${env.BID}: build_args must include KDIR")
                                    }

                                    echo "Build hvisor-tool [BID=${env.BID}, TARCH=${tarch}, KDIR=${kdir}]"
                                    if (!fileExists(env.HVISOR_TOOL_PATH)) {
                                        sh "mkdir -p ${env.HVISOR_TOOL_PATH}"
                                    }
                                    dir(env.HVISOR_TOOL_PATH) {
                                        checkout([
                                            $class: 'GitSCM',
                                            branches: [[name: '*/main']],
                                            extensions: [[$class: 'CloneOption', depth: 1, noTags: true]],
                                            userRemoteConfigs: [[url: env.HVISOR_TOOL_URL]]
                                        ])
                                    }
                                    sh """
                                        export PATH=${env.TOOLCHAIN_PATHS}:\$PATH
                                        cd ${env.HVISOR_TOOL_PATH}
                                        make all ARCH=${tarch} KDIR=${kdir}
                                    """
                                }
                            }
                        }
                    }

                    stage('Prepare test') {
                        when {
                            expression {
                                return getBidConfig(loadCiYaml(), env.BID) != null
                            }
                        }
                        steps {
                            dir(matrixCellDir()) {
                                script {
                                    def ci = loadCiYaml()
                                    def bidCfg = getBidConfig(ci, env.BID)
                                    def buildArgs = parseCiBuildArgs(bidCfg)
                                    def bidPrepare = parseBid(env.BID)
                                    def arch = bidPrepare.arch
                                    def board = bidPrepare.board
                                    def kdir = (buildArgs.KDIR ?: '').toString()
                                    def testsCfg = bidCfg.tests ?: [:]
                                    def mode = (testsCfg.mode ?: '').toString().trim()
                                    if (!kdir || !mode) {
                                        error("jenkins/ci.yaml BID=${env.BID}: tests.mode and build_args KDIR are required")
                                    }

                                    if (mode == 'qemu') {
                                        def prepareScript = "jenkins/prepare.sh"
                                        def externalFile = "${env.TEST_IMG_BASE}/${arch}/${board}"
                                        def configure = "./platform/${arch}/${board}/"
                                        echo "Prepare rootfs [BID=${env.BID}]"
                                        sh """
                                            cp -r ${externalFile}/* ${configure}
                                            chmod +x "${prepareScript}"
                                            sudo -E env \\
                                                ARCH="${arch}" \\
                                                BOARD="${board}" \\
                                                KDIR="${kdir}" \\
                                                WORKSPACE_ROOT="\$(pwd)" \\
                                                HVISOR_TOOL_PATH="${env.HVISOR_TOOL_PATH}" \\
                                                "${prepareScript}"
                                        """
                                    } else if (mode == 'board') {
                                        // Placeholder for future board artifact distribution by network.
                                        echo "Board prepare placeholder [BID=${env.BID}]"
                                    } else {
                                        error("jenkins/ci.yaml BID=${env.BID}: unsupported tests.mode='${mode}'")
                                    }
                                }
                            }
                        }
                    }

                    stage('Run test cases') {
                        when {
                            expression {
                                return getBidConfig(loadCiYaml(), env.BID) != null
                            }
                        }
                        steps {
                            dir(matrixCellDir()) {
                                script {
                                    def bidRun = parseBid(env.BID)
                                    def tArch = bidRun.arch
                                    def tBoard = bidRun.board
                                    echo "Run tests via ci_runner [BID=${env.BID}]"
                                    sh """
                                        export TERM=\${TERM:-xterm}
                                        export PATH=${env.CARGO_HOME}/bin:${env.TOOLCHAIN_PATHS}:\$PATH
                                        export PATH=${env.QEMU_PATH}:\$PATH
                                        ${kconfigSetupShell(tArch, tBoard)}
                                        python3 jenkins/ci_runner.py \
                                            --bid "${env.BID}"
                                    """
                                }
                            }
                        }
                    }
                }

                post {
                    success {
                        script { publishMatrixCheckCompleted('SUCCESS') }
                    }
                    failure {
                        script { publishMatrixCheckCompleted('FAILURE') }
                    }
                    unstable {
                        script { publishMatrixCheckCompleted('FAILURE') }
                    }
                }
            }
        }
    }
}
