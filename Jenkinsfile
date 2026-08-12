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

/** Isolated workspace for top-level CI jobs (linter, license-checker, …). */
def jenkinsJobDir(String name) {
    return "${env.WORKSPACE}/.jenkins/${name}"
}

/** Copy repo into an isolated dir; excludes other Jenkins sandboxes from the source tree. */
def syncWorkspaceTo(String destDir) {
    sh """
        mkdir -p '${destDir}'
        rsync -a --delete \\
            --exclude '.jenkins/' \\
            --exclude '.matrix/' \\
            --exclude '.jenkins-matrix/' \\
            '${env.WORKSPACE}/' '${destDir}/'
    """
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

/** Marker file: only finish checks that published IN_PROGRESS. */
def githubCheckStartedMarker(String checkName) {
    def safe = checkName.replace('/', '__').replaceAll(/[^A-Za-z0-9_.-]/, '_')
    return "${env.WORKSPACE}/.jenkins/check-started/${safe}"
}

def markGithubCheckStarted(String checkName) {
    def marker = githubCheckStartedMarker(checkName)
    sh "mkdir -p '${env.WORKSPACE}/.jenkins/check-started' && : > '${marker}'"
}

def isGithubCheckStarted(String checkName) {
    return fileExists(githubCheckStartedMarker(checkName))
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
    markGithubCheckStarted(checkName)
}

def publishGithubCheckCompleted(String checkName, String conclusion) {
    def summaries = [
        'SUCCESS'  : 'Passed',
        'FAILURE'  : 'Failed',
        'CANCELLED': 'Cancelled',
    ]
    publishChecks(
        name: checkName,
        title: checkName,
        summary: summaries.get(conclusion, conclusion),
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

def finishGithubCheck(String checkName, String buildResult) {
    if (!isGithubCheckStarted(checkName)) {
        echo "Skip GitHub check completion for '${checkName}' (in-progress was never published)"
        return
    }
    def conclusion = [
        'SUCCESS' : 'SUCCESS',
        'FAILURE' : 'FAILURE',
        'UNSTABLE': 'FAILURE',
        'ABORTED' : 'CANCELLED',
        'NOT_BUILT': 'CANCELLED',
    ].get(buildResult ?: '', 'FAILURE')
    publishGithubCheckCompleted(checkName, conclusion)
}

def hasCiTests() {
    return getBidConfig(loadCiYaml(), env.BID) != null
}

def toolchainPathShell() {
    return "export PATH=${env.CARGO_HOME}/bin:${env.TOOLCHAIN_PATHS}:\$PATH"
}

def qemuPathShell() {
    return "export PATH=${env.QEMU_PATH}:\$PATH"
}

/** Kconfig: symlink agent venv + make defconfig. Keep in sync with Makefile kconfig_python path. */
def kconfigSetupShell(String arch, String board) {
    return """
        ${toolchainPathShell()}
        chmod +x tools/kconfig/host_config.sh tools/kconfig/save_defconfig.sh 2>/dev/null || true
        if [ ! -x tools/kconfig/.venv/bin/python ]; then
            if [ ! -x ${env.KCONFIG_VENV}/bin/python ]; then
                echo "ERROR: CI kconfig venv missing: ${env.KCONFIG_VENV}/bin/python" >&2
                exit 1
            fi
            mkdir -p tools/kconfig
            ln -sfn ${env.KCONFIG_VENV} tools/kconfig/.venv
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
            deleteDir()
        }
    }

    environment {
        HVISOR_TOOL_URL = 'https://github.com/syswonder/hvisor-tool.git'
        HVISOR_TOOL_PATH = 'hvisor-tool'
        RUST_HOME = '/usr/local/rustup'
        CARGO_HOME = '/usr/local/cargo'
        QEMU_PATH = '/home/light/DEMO/qemu-10.1.0/build'
        TEST_IMG_BASE = '/home/light/DEMO/syswonder/test_img'
        KCONFIG_VENV = "/home/light/DEMO/syswonder/test_img/venv"
        RISCV_TOOLCHAIN_PATH = '/home/light/DEMO/toolchain/riscv64-glibc-ubuntu-24.04-gcc'
        AARCH64_TOOLCHAIN_PATH = '/home/light/DEMO/toolchain/gcc-arm-10.3-2021.07-x86_64-aarch64-none-linux-gnu'
        LOONGARCH64_TOOLCHAIN_PATH = '/home/light/DEMO/toolchain/loongarch_cross_tools'
        // All toolchain bins on PATH; same for every matrix cell (no per-arch selection).
        TOOLCHAIN_PATHS = "${env.RISCV_TOOLCHAIN_PATH}/bin:${env.AARCH64_TOOLCHAIN_PATH}/bin:${env.LOONGARCH64_TOOLCHAIN_PATH}/bin"
    }

    stages {
        stage('Linter') {
            steps {
                script {
                    publishGithubCheckInProgress('linter')
                    def cellWs = jenkinsJobDir('linter')
                    syncWorkspaceTo(cellWs)
                    dir(cellWs) {
                        sh kconfigSetupShell('aarch64', 'qemu-gicv3') + '''
                            cargo fmt --all -- --check
                        '''
                    }
                }
            }
            post {
                always {
                    script { finishGithubCheck('linter', currentBuild.currentResult) }
                }
            }
        }

        stage('License checker') {
            steps {
                script {
                    publishGithubCheckInProgress('license-checker')
                    def cellWs = jenkinsJobDir('license-checker')
                    syncWorkspaceTo(cellWs)
                    dir(cellWs) {
                        sh """
                            chmod +x tools/license_checker.sh
                            ./tools/license_checker.sh
                        """
                    }
                }
            }
            post {
                always {
                    script { finishGithubCheck('license-checker', currentBuild.currentResult) }
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
                            'aarch64/jeston-orin',
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
                    stage('Prepare cell') {
                        steps {
                            script {
                                publishMatrixCheckInProgress()
                                syncWorkspaceTo(matrixCellDir())
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
                                            ${toolchainPathShell()}
                                            make dtb ARCH=${arch} BOARD=${board}
                                        """
                                    }
                                    sh """
                                        ${toolchainPathShell()}
                                        make all ARCH=${arch} BOARD=${board} MODE=release
                                    """
                                }
                            }
                        }
                    }

                    stage('Build hvisor-tool') {
                        when {
                            expression { return hasCiTests() }
                        }
                        steps {
                            dir(matrixCellDir()) {
                                script {
                                    def bidCfg = getBidConfig(loadCiYaml(), env.BID)
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
                                        make -C ${env.HVISOR_TOOL_PATH} all ARCH=${tarch} KDIR=${kdir}
                                    """
                                }
                            }
                        }
                    }

                    stage('Prepare test') {
                        when {
                            expression { return hasCiTests() }
                        }
                        steps {
                            dir(matrixCellDir()) {
                                script {
                                    def bidCfg = getBidConfig(loadCiYaml(), env.BID)
                                    def buildArgs = parseCiBuildArgs(bidCfg)
                                    def bidParsed = parseBid(env.BID)
                                    def arch = bidParsed.arch
                                    def board = bidParsed.board
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
                            expression { return hasCiTests() }
                        }
                        steps {
                            dir(matrixCellDir()) {
                                script {
                                    echo "Run tests via ci_runner [BID=${env.BID}]"
                                    sh """
                                        export TERM=\${TERM:-xterm}
                                        ${toolchainPathShell()}
                                        ${qemuPathShell()}
                                        python3 jenkins/ci_runner.py \
                                            --bid "${env.BID}"
                                    """
                                }
                            }
                        }
                    }
                }

                post {
                    always {
                        script { finishGithubCheck(matrixCheckName(), currentBuild.currentResult) }
                    }
                }
            }
        }
    }
}
