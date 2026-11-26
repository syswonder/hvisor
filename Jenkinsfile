import org.jenkinsci.plugins.workflow.steps.FlowInterruptedException

/** ARCH/BOARD come from matrix BID (arch/board); ci.yaml must not duplicate them. */
def parseBid(String bid) {
    def parts = (bid ?: '').split('/', 2)
    if (parts.size() != 2 || !parts[0] || !parts[1]) {
        error("invalid BID: ${bid}")
    }
    return [arch: parts[0], board: parts[1]]
}

/** Optional platform_board in ci.yaml maps BID board to make/platform board. */
def platformBoard(String bid, String configuredBoard = '') {
    return configuredBoard ?: parseBid(bid).board
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
    return "${env.WORKSPACE}/.jenkins/check-started/${env.BUILD_NUMBER}/${safe}"
}

def markGithubCheckStarted(String checkName) {
    def marker = githubCheckStartedMarker(checkName)
    sh "mkdir -p '${env.WORKSPACE}/.jenkins/check-started/${env.BUILD_NUMBER}' && : > '${marker}'"
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
        make defconfig ARCH=${arch} BOARD=${board} BID=
    """
}

/** Keep the curated CI BID inventory separate from platform-directory discovery. */
def matrixBids() {
    return [
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
        'x86_64/qemu_asterinas',
    ]
}

def compileMatrixCell() {
    def bid = parseBid(env.BID)
    def arch = bid.arch
    // BID may name a test variant (qemu_asterinas); platform_board selects the actual make/platform board (qemu).
    def bidCfg = getBidConfig(loadCiYaml(), env.BID)
    def board = platformBoard(env.BID, bidCfg?.platform_board?.toString() ?: '')
    echo "Compile hvisor [BID=${env.BID}, ARCH=${arch}, BOARD=${board}]"
    sh kconfigSetupShell(arch, board)
    if (arch != 'x86_64') {
        sh """
            ${toolchainPathShell()}
            make dtb ARCH=${arch} BOARD=${board} BID=
        """
    }
    sh """
        ${toolchainPathShell()}
        make all ARCH=${arch} BOARD=${board} MODE=release BID=
    """
}

def buildMatrixTool() {
    def bidCfg = getBidConfig(loadCiYaml(), env.BID)
    def buildArgs = parseCiBuildArgs(bidCfg)
    def bidTool = parseBid(env.BID)
    def tarch = normalizeToolArch(buildArgs.TARCH ?: bidTool.arch)
    def kdir = buildArgs.KDIR
    if (!kdir) {
        error("jenkins/ci.yaml BID=${env.BID}: build_args must include KDIR")
    }

    echo "Build in-tree hvisor-tool [BID=${env.BID}, TARCH=${tarch}, KDIR=${kdir}]"
    sh """
        export PATH=${env.TOOLCHAIN_PATHS}:\$PATH
        make -C ${env.HVISOR_TOOL_PATH} all ARCH=${tarch} KDIR=${kdir}
    """
}

def prepareMatrixTest() {
    def bidCfg = getBidConfig(loadCiYaml(), env.BID)
    def buildArgs = parseCiBuildArgs(bidCfg)
    def bidParsed = parseBid(env.BID)
    def arch = bidParsed.arch
    // Keep the server artifact BID separate from the real platform board used by make and prepare.sh.
    def board = platformBoard(env.BID, bidCfg?.platform_board?.toString() ?: '')
    def kdir = (buildArgs.KDIR ?: '').toString()
    def testsCfg = bidCfg.tests ?: [:]
    def artifactDir = testsCfg.artifact_dir ?: bidParsed.board
    def mode = (testsCfg.mode ?: '').toString().trim()
    if (!kdir || !mode) {
        error("jenkins/ci.yaml BID=${env.BID}: tests.mode and build_args KDIR are required")
    }

    if (mode == 'qemu') {
        def prepareScript = "jenkins/prepare.sh"
        def externalFile = "${env.TEST_IMG_BASE}/${arch}/${artifactDir}"
        def configure = "./platform/${arch}/${board}/"
        echo "Prepare rootfs [BID=${env.BID}]"
        sh """
            cp -r ${externalFile}/* ${configure}
            if [ "${artifactDir}" != "${board}" ]; then
                mkdir -p "${configure}/image/kernel" "${configure}/image/virtdisk"
                cp "${env.TEST_IMG_BASE}/${arch}/${board}/image/kernel/setup.bin" "${configure}/image/kernel/setup.bin"
                cp "${env.TEST_IMG_BASE}/${arch}/${board}/image/kernel/vmlinux.bin" "${configure}/image/kernel/vmlinux.bin"
                cp "${env.TEST_IMG_BASE}/${arch}/${board}/image/virtdisk/rootfs1.img" "${configure}/image/virtdisk/rootfs1.img"
            fi
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
            ${toolchainPathShell()}
            tftp_staging="\$(pwd)/.tftp-staging"
            rm -rf "\${tftp_staging}"
            make cp ARCH=${arch} BOARD=${board} MODE=release TFTP_DIR="\${tftp_staging}"
            test -f "\${tftp_staging}/hvisor.bin"
            sudo mkdir -p "${tftpDir}"
            sudo find "${tftpDir}" -mindepth 1 -maxdepth 1 -type f -delete
            sudo cp "\${tftp_staging}/hvisor.bin" "${tftpDir}/"
            test -f "${tftpDir}/hvisor.bin" || {
                echo "error: hvisor.bin missing in ${tftpDir}" >&2
                exit 1
            }
        """
        zone0Dtbs.each { dtb ->
            sh """
                test -f "${dtb}"
                sudo cp "${dtb}" "${tftpDir}/"
            """
        }
        sh """
            test -f "${zone0Image}" || {
                echo "error: zone0 kernel Image not found: ${zone0Image}" >&2
                exit 1
            }
            sudo cp "${zone0Image}" "${tftpDir}/Image"
            test -f "${tftpDir}/Image" || {
                echo "error: Image missing in ${tftpDir}" >&2
                exit 1
            }
            sudo chmod -R a+rX "${tftpDir}"
            ls -la "${tftpDir}"
        """
    } else {
        error("jenkins/ci.yaml BID=${env.BID}: unsupported tests.mode='${mode}'")
    }
}

def runMatrixTests() {
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

/** A slot covers the entire cell: tests may invoke make ci-run again. */
def runMatrixCell(String bid, Map profile) {
    def result = 'SUCCESS'
    withEnv([
        "BID=${bid}",
        "CARGO_BUILD_JOBS=${profile.cargo_jobs}",
        'MAKEFLAGS=-j1',
    ]) {
        try {
            stage("${bid} / Prepare cell") {
                publishMatrixCheckInProgress()
                syncWorkspaceTo(matrixCellDir())
            }
            dir(matrixCellDir()) {
                stage("${bid} / Compile") {
                    compileMatrixCell()
                }
                if (hasCiTests()) {
                    stage("${bid} / Build hvisor-tool") {
                        buildMatrixTool()
                    }
                    stage("${bid} / Prepare test") {
                        prepareMatrixTest()
                    }
                    stage("${bid} / Run test cases") {
                        runMatrixTests()
                    }
                }
            }
        } catch (FlowInterruptedException interrupted) {
            result = 'ABORTED'
            throw interrupted
        } catch (Exception failure) {
            result = 'FAILURE'
            echo "CI_CELL_FAILURE [BID=${bid}]: ${failure.message}"
        } finally {
            try {
                dir(matrixCellDir()) {
                    archiveArtifacts artifacts: 'logs/**', allowEmptyArchive: true
                }
            } catch (FlowInterruptedException interrupted) {
                result = 'ABORTED'
                throw interrupted
            } catch (Exception failure) {
                if (result != 'ABORTED') {
                    result = 'FAILURE'
                }
                echo "CI_CELL_ARCHIVE_FAILURE [BID=${bid}]: ${failure.message}"
            } finally {
                try {
                    // Never use the aggregate currentBuild result for another cell.
                    finishGithubCheck(bid, result)
                } catch (FlowInterruptedException interrupted) {
                    result = 'ABORTED'
                    throw interrupted
                } catch (Exception failure) {
                    if (result != 'ABORTED') {
                        result = 'FAILURE'
                    }
                    echo "CI_CELL_CHECK_FAILURE [BID=${bid}]: ${failure.message}"
                }
            }
        }
    }
    return result
}

def ciResourceDir() {
    return ".jenkins/ci-resources/${env.BUILD_NUMBER}"
}

/** Each worker takes the next queued BID immediately after releasing its slot. */
def runMatrixWorker(String worker, String stateFile, Map setup) {
    while (true) {
        def claim = readJSON returnPojo: true, text: sh(
            returnStdout: true,
            script: "python3 jenkins/ci_scheduler.py claim --state '${stateFile}' --worker '${worker}'",
        ).trim()
        if (claim.status == 'done') {
            return
        }
        if (claim.status == 'stopped') {
            echo "${worker}: ${claim.reason}"
            return
        }
        if (claim.status == 'wait') {
            echo "CI_RESOURCE_WAIT [${worker}]: ${claim.reason}; available=${claim.available_mib} MiB"
            sleep time: setup.policy.poll_seconds as int, unit: 'SECONDS'
            continue
        }
        if (claim.status != 'run') {
            error("Unexpected CI scheduler response: ${claim.status}")
        }

        String bid = claim.bid
        String result = 'ABORTED'
        try {
            stage("Matrix - BID = '${bid}'") {
                echo "CI_CELL_START [BID=${bid}, worker=${worker}, active=${claim.active}, cargo_jobs=${setup.profile.cargo_jobs}]"
                result = runMatrixCell(bid, setup.profile)
                if (result != 'SUCCESS') {
                    // Display a failed cell without cancelling queued cells.
                    catchError(buildResult: 'SUCCESS', stageResult: 'FAILURE', catchInterruptions: false) {
                        error("CI cell ${bid}: ${result}")
                    }
                }
            }
        } catch (FlowInterruptedException interrupted) {
            result = 'ABORTED'
            throw interrupted
        } catch (Exception failure) {
            result = 'FAILURE'
            echo "CI_CELL_FAILURE [BID=${bid}]: ${failure.message}"
        } finally {
            sh "python3 jenkins/ci_scheduler.py complete --state '${stateFile}' --worker '${worker}' --bid '${bid}' --result '${result}'"
            echo "CI_CELL_END [BID=${bid}, worker=${worker}, result=${result}]"
        }
    }
}

def runBoundedMatrix() {
    def bids = matrixBids()
    // A configured test (including a BID alias) must never silently disappear.
    for (def entry : loadCiYaml().bids) {
        def bid = (entry?.bid ?: '').toString().trim()
        if (!bids.contains(bid)) {
            error("jenkins/ci.yaml BID is missing from CI matrix: ${bid}")
        }
    }

    String resourceDir = ciResourceDir()
    String stateFile = "${resourceDir}/state.json"
    String stopFile = "${resourceDir}/monitor.stop"
    String metricsFile = "${resourceDir}/metrics.jsonl"
    sh "mkdir -p '${resourceDir}'"
    writeJSON file: "${resourceDir}/bids.json", json: bids

    def outcome
    def report
    try {
        def setup = readJSON returnPojo: true, text: sh(
            returnStdout: true,
            script: "python3 jenkins/ci_scheduler.py init --state '${stateFile}' --policy jenkins/resource_policy.json --bids '${resourceDir}/bids.json'",
        ).trim()
        echo "CI_RESOURCE_PROFILE: ${setup.profile.name}, slots=${setup.profile.slots}, cargo_jobs=${setup.profile.cargo_jobs}, validated=${setup.profile.validated}, available=${setup.available_mib} MiB"
        def workers = [:]
        for (int slot = 1; slot <= (setup.profile.slots as int); slot++) {
            String worker = "slot-${slot}"
            workers[worker] = {
                runMatrixWorker(worker, stateFile, setup)
            }
        }
        // Ordinary failures are recorded per BID. Unexpected interruptions
        // stop both the worker group and the foreground resource sampler.
        parallel(
            'platform-workers': {
                try {
                    parallel workers
                } finally {
                    writeFile file: stopFile, text: 'done\n'
                }
            },
            'resource-monitor': {
                int rc = sh returnStatus: true,
                    script: "python3 jenkins/ci_scheduler.py monitor --state '${stateFile}' --output '${metricsFile}' --stop-file '${stopFile}'"
                if (rc != 0) {
                    // Do not start new cells without observations. Already
                    // running tests can finish their normal cleanup.
                    sh "python3 jenkins/ci_scheduler.py stop --state '${stateFile}' --reason 'CI_RESOURCE_MONITOR_FAILED'"
                }
            },
            failFast: true,
        )
    } finally {
        if (fileExists(stateFile)) {
            def resultJson = sh(
                returnStdout: true,
                script: "python3 jenkins/ci_scheduler.py finalize --state '${stateFile}'",
            ).trim()
            writeFile file: "${resourceDir}/results.json", text: resultJson + '\n'
            outcome = readJSON text: resultJson, returnPojo: true
            // A suspended/aborted queue must not leave unrun BIDs as successes.
            for (String bid : outcome.unfinished_bids) {
                try {
                    publishGithubCheckCompleted(bid, 'CANCELLED')
                } catch (FlowInterruptedException interrupted) {
                    throw interrupted
                } catch (Exception failure) {
                    echo "CI_CELL_CHECK_FAILURE [BID=${bid}]: ${failure.message}"
                }
            }
            def metricsJson = sh(
                returnStdout: true,
                script: "python3 jenkins/ci_scheduler.py summarize --state '${stateFile}' --metrics '${metricsFile}'",
            ).trim()
            writeFile file: "${resourceDir}/resource-summary.json", text: metricsJson + '\n'
            report = readJSON text: metricsJson, returnPojo: true
            echo "CI_RESOURCE_SUMMARY: ${metricsJson}"
            echo "CI_MATRIX_RESULT: ${outcome.counts}; total=${outcome.total}, peak_active=${outcome.peak_active}"
        }
    }
    if (!outcome?.success) {
        error("CI matrix incomplete or failed: ${outcome?.counts}; ${outcome?.stop_reason ?: ''}")
    }
    if (!report?.samples || report.malformed_samples || report.resource_incident) {
        error("CI resource observation incomplete or unsafe: ${report}")
    }
    if (!report.meets_reserve) {
        echo "CI_RESOURCE_WARNING: reserve target was not maintained; do not promote this profile"
    }
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
            steps {
                script { runBoundedMatrix() }
            }
            post {
                always {
                    script {
                        archiveArtifacts artifacts: "${ciResourceDir()}/**",
                            excludes: '**/*.lock', allowEmptyArchive: true
                    }
                }
            }
        }
    }
}
