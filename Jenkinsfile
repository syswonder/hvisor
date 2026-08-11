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

/** Stable directory name for a BID under ``$WORKSPACE/logs/``. */
def bidLogKey(String bid = env.BID) {
    return (bid ?: '').replace('/', '__')
}

/** Copy per-cell logs out to ``$WORKSPACE/logs/<bid>`` before ``.matrix`` is removed. */
def collectCellLogs() {
    def src = "${matrixCellDir()}/logs"
    def dest = "${env.WORKSPACE}/logs/${bidLogKey()}"
    def logsRoot = "${env.WORKSPACE}/logs"
    sh """
        if [ ! -d '${src}' ]; then
            echo "No cell logs to collect: ${src}"
            exit 0
        fi
        mkdir -p '${dest}' 2>/dev/null || sudo mkdir -p '${dest}'
        sudo chown -R "\$(id -u):\$(id -g)" '${logsRoot}' 2>/dev/null || true
        # Source may be root-owned after board sudo ci_runner.
        if ! cp -a '${src}/.' '${dest}/' 2>/dev/null; then
            sudo cp -a '${src}/.' '${dest}/'
            sudo chown -R "\$(id -u):\$(id -g)" '${dest}'
        fi
        echo "Collected cell logs -> ${dest}"
        ls -la '${dest}' || true
    """
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
            --exclude 'logs/' \\
            --exclude '__pycache__/' \\
            --exclude '*.pyc' \\
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

def loadHvisorToolConfig() {
    def path = 'jenkins/hvisor-tool.yaml'
    def defaults = [
        repo: env.HVISOR_TOOL_URL ?: 'https://github.com/syswonder/hvisor-tool.git',
        ref : 'main',
    ]
    if (!fileExists(path)) {
        return defaults
    }
    def data = readYaml file: path
    def repo = (data?.repo ?: defaults.repo).toString().trim()
    def ref = (data?.ref ?: defaults.ref).toString().trim()
    if (!repo || !ref) {
        error("${path}: 'repo' and 'ref' must be non-empty")
    }
    return [repo: repo, ref: ref]
}

def checkoutHvisorTool(Map toolCfg, String targetDir) {
    def url = toolCfg.repo.toString().trim()
    def ref = (toolCfg.ref ?: '').trim()

    if (ref ==~ /^[0-9a-fA-F]{7,40}$/) {
        sh """
            set -eux
            rm -rf '${targetDir}'
            git init '${targetDir}'
            git -C '${targetDir}' remote add origin '${url}'
            git -C '${targetDir}' fetch --depth 50 origin '${ref}'
            git -C '${targetDir}' checkout FETCH_HEAD
        """
    } else if (ref.startsWith('refs/')) {
        sh """
            set -eux
            rm -rf '${targetDir}'
            git init '${targetDir}'
            git -C '${targetDir}' remote add origin '${url}'
            git -C '${targetDir}' fetch --depth 1 origin '${ref}:ci-ref'
            git -C '${targetDir}' checkout ci-ref
        """
    } else {
        sh """
            set -eux
            rm -rf '${targetDir}'
            git clone --depth 1 --branch '${ref}' '${url}' '${targetDir}'
        """
    }
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
            // Keep $WORKSPACE/logs (collected from cells); only tear down sandboxes.
            sh 'sudo rm -rf .matrix 2>/dev/null || rm -rf .matrix || true'
            archiveArtifacts artifacts: 'logs/**/*', allowEmptyArchive: true
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
        TFTP_DIR = '/home/light/tftp'
        PYTHONDONTWRITEBYTECODE = '1'
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

        stage('Checkout hvisor-tool') {
            steps {
                script {
                    def toolCfg = loadHvisorToolConfig()
                    echo "Checkout hvisor-tool [repo=${toolCfg.repo}, ref=${toolCfg.ref}]"
                    checkoutHvisorTool(toolCfg, env.HVISOR_TOOL_PATH)
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
                                        def tftpDir = (testsCfg.tftp_dir ?: env.TFTP_DIR).toString()
                                        def zone0Dtbs = testsCfg.zone0_dtbs ?: []
                                        if (testsCfg.zone0_dtb) {
                                            zone0Dtbs = [testsCfg.zone0_dtb]
                                        }
                                        def zone0Image = (testsCfg.zone0_image ?: "${kdir}/arch/arm64/boot/Image").toString()
                                        echo "Deploy TFTP artifacts [BID=${env.BID}, TFTP_DIR=${tftpDir}]"
                                        sh """
                                            export TERM=\${TERM:-xterm}
                                            sudo mkdir -p "${tftpDir}"
                                            sudo find "${tftpDir}" -mindepth 1 -maxdepth 1 -type f -delete
                                            sudo make cp ARCH=${arch} BOARD=${board} MODE=release TFTP_DIR="${tftpDir}"
                                        """
                                        zone0Dtbs.each { dtb ->
                                            sh """
                                                test -f "${dtb}"
                                                sudo cp "${dtb}" "${tftpDir}/"
                                            """
                                        }
                                        sh """
                                            test -f "${zone0Image}"
                                            sudo cp "${zone0Image}" "${tftpDir}/Image"
                                            sudo chmod -R a+rX "${tftpDir}"
                                            ls -la "${tftpDir}"
                                        """
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
                                    def bidCfg = getBidConfig(loadCiYaml(), env.BID)
                                    def mode = (bidCfg.tests?.mode ?: '').toString().trim()
                                    echo "Run tests via ci_runner [BID=${env.BID}, mode=${mode}]"
                                    if (mode == 'board') {
                                        sh """
                                            export TERM=\${TERM:-xterm}
                                            sudo -E python3 jenkins/ci_runner.py \
                                                --bid "${env.BID}"
                                        """
                                    } else {
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
                }

                post {
                    always {
                        script {
                            collectCellLogs()
                            finishGithubCheck(matrixCheckName(), currentBuild.currentResult)
                        }
                    }
                }
            }
        }

        stage('hvisor-tool on the branch of test') {
            steps {
                script {
                    def checkName = 'hvisor-tool on the branch of test'
                    publishGithubCheckInProgress(checkName)
                    def ref = loadHvisorToolConfig().ref
                    echo "Verify hvisor-tool pin [ref=${ref}]"
                    if (ref != 'main') {
                        error("hvisor-tool ref must be main for merge (current ref=${ref}). Restore ref: main in jenkins/hvisor-tool.yaml.")
                    }
                }
            }
            post {
                always {
                    script { finishGithubCheck('hvisor-tool on the branch of test', currentBuild.currentResult) }
                }
            }
        }
    }
}
