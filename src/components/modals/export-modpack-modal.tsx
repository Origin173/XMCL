import {
  Box,
  Button,
  Card,
  Center,
  Flex,
  HStack,
  Icon,
  Modal,
  ModalBody,
  ModalCloseButton,
  ModalContent,
  ModalFooter,
  ModalHeader,
  ModalOverlay,
  ModalProps,
  NumberInput,
  NumberInputField,
  Progress,
  Step,
  StepIcon,
  StepIndicator,
  StepNumber,
  StepSeparator,
  StepStatus,
  StepTitle,
  Stepper,
  Switch,
  Text,
  VStack,
  useSteps,
} from "@chakra-ui/react";
import { listen } from "@tauri-apps/api/event";
import { save } from "@tauri-apps/plugin-dialog";
import React, {
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import { useTranslation } from "react-i18next";
import { LuArchive, LuPackage, LuPackage2 } from "react-icons/lu";
import { BeatLoader } from "react-spinners";
import Editable from "@/components/common/editable";
import FileTreeSelector, {
  defaultSelectedFromEntries,
} from "@/components/common/file-tree-selector";
import { OptionItemGroup } from "@/components/common/option-item";
import { useLauncherConfig } from "@/contexts/config";
import { useToast } from "@/contexts/toast";
import {
  ExportFileEntry,
  ExportFormat,
  ExportModpackMeta,
} from "@/models/instance/misc";
import { InstanceService } from "@/services/instance";

type ExportStage = "matching" | "packing" | "writingManifest" | "done";

interface ExportProgressPayload {
  current: number;
  total: number;
  fileName: string;
  stage: ExportStage;
}

interface ExportModpackModalProps extends Omit<ModalProps, "children"> {
  instanceId: string;
  instanceName?: string;
}

const FORMAT_OPTIONS = [
  {
    format: ExportFormat.Modrinth,
    translationKey: "modrinth",
    icon: LuPackage,
    ext: ".mrpack",
    descriptionKey: "ExportModpackModal.format.modrinth.description",
  },
  {
    format: ExportFormat.CurseForge,
    translationKey: "curseforge",
    icon: LuPackage2,
    ext: ".zip",
    descriptionKey: "ExportModpackModal.format.curseforge.description",
  },
  {
    format: ExportFormat.MultiMC,
    translationKey: "multimc",
    icon: LuArchive,
    ext: ".zip",
    descriptionKey: "ExportModpackModal.format.multimc.description",
  },
];

const ExportModpackModal: React.FC<ExportModpackModalProps> = ({
  instanceId,
  instanceName,
  ...modalProps
}) => {
  const { t } = useTranslation();
  const { config } = useLauncherConfig();
  const primaryColor = config.appearance.theme.primaryColor;
  const toast = useToast();

  const { activeStep, setActiveStep } = useSteps({ index: 0, count: 3 });

  // Step 1 – format
  const [selectedFormat, setSelectedFormat] = useState<ExportFormat>(
    ExportFormat.Modrinth
  );

  // Step 2 – meta
  const [metaName, setMetaName] = useState(instanceName ?? "");
  const [metaAuthor, setMetaAuthor] = useState("");
  const [metaVersion, setMetaVersion] = useState("1.0.0");
  const [metaDescription, setMetaDescription] = useState("");

  // Export options
  const [noCreateRemoteFiles, setNoCreateRemoteFiles] = useState(false);
  const [skipCurseForgeRemoteFiles, setSkipCurseForgeRemoteFiles] =
    useState(false);
  const [minMemory, setMinMemory] = useState("512");

  // Step 3 – file selection
  const [fileEntries, setFileEntries] = useState<ExportFileEntry[]>([]);
  const [selectedFiles, setSelectedFiles] = useState<Set<string>>(new Set());
  const [isLoadingFiles, setIsLoadingFiles] = useState(false);

  const [isExporting, setIsExporting] = useState(false);

  const [exportProgress, setExportProgress] =
    useState<ExportProgressPayload | null>(null);
  const unlistenRef = useRef<(() => void) | null>(null);

  useEffect(() => {
    return () => {
      unlistenRef.current?.();
    };
  }, []);

  // Load file entries when reaching step 3
  useEffect(() => {
    if (activeStep !== 2 || fileEntries.length > 0) return;
    setIsLoadingFiles(true);
    InstanceService.scanInstanceFilesForExport(instanceId)
      .then((res) => {
        if (res.status === "success") {
          setFileEntries(res.data);
          setSelectedFiles(defaultSelectedFromEntries(res.data));
        } else {
          toast({
            title: res.message,
            description: res.details,
            status: "error",
          });
        }
      })
      .finally(() => setIsLoadingFiles(false));
  }, [activeStep, instanceId, fileEntries.length, toast]);

  const handleExport = useCallback(async () => {
    const ext = selectedFormat === ExportFormat.Modrinth ? "mrpack" : "zip";
    const outputPath = await save({
      filters: [
        { name: t("ExportModpackModal.dialog.filterName"), extensions: [ext] },
      ],
      defaultPath: `${metaName}.${ext}`,
    });
    if (!outputPath) return;

    setIsExporting(true);
    setExportProgress(null);

    const unlisten = await listen<ExportProgressPayload>(
      "export-modpack:progress",
      (event) => {
        setExportProgress(event.payload);
      }
    );
    unlistenRef.current = unlisten;

    const meta: ExportModpackMeta = {
      name: metaName,
      author: metaAuthor,
      version: metaVersion,
      description: metaDescription || undefined,
      noCreateRemoteFiles: noCreateRemoteFiles || undefined,
      skipCurseForgeRemoteFiles: skipCurseForgeRemoteFiles || undefined,
      minMemory:
        selectedFormat === ExportFormat.MultiMC
          ? parseInt(minMemory, 10) || undefined
          : undefined,
    };

    const res = await InstanceService.exportModpack(
      instanceId,
      selectedFormat,
      meta,
      Array.from(selectedFiles),
      outputPath
    );

    unlisten();
    unlistenRef.current = null;
    setIsExporting(false);
    setExportProgress(null);

    if (res.status === "success") {
      toast({
        title: t("ExportModpackModal.toast.success"),
        status: "success",
      });
      modalProps.onClose();
    } else {
      toast({ title: res.message, description: res.details, status: "error" });
    }
  }, [
    selectedFormat,
    metaName,
    metaAuthor,
    metaVersion,
    metaDescription,
    noCreateRemoteFiles,
    skipCurseForgeRemoteFiles,
    minMemory,
    instanceId,
    selectedFiles,
    modalProps,
    toast,
    t,
  ]);

  // ─── Step 1: Format selection ────────────────────────────────────────────────
  const step1Content = useMemo(
    () => (
      <>
        <ModalBody>
          <VStack spacing={3} pt={2}>
            {FORMAT_OPTIONS.map(
              ({ format, translationKey, icon, ext, descriptionKey }) => (
                <Card
                  key={format}
                  w="100%"
                  p={4}
                  cursor="pointer"
                  borderWidth={2}
                  borderColor={
                    selectedFormat === format
                      ? `${primaryColor}.400`
                      : "transparent"
                  }
                  onClick={() => setSelectedFormat(format)}
                  _hover={{ borderColor: `${primaryColor}.300` }}
                >
                  <HStack spacing={3}>
                    <Icon as={icon} boxSize={6} color={`${primaryColor}.400`} />
                    <Box flex={1}>
                      <Text fontWeight="bold" fontSize="sm">
                        {t(`ExportModpackModal.format.${translationKey}.name`)}
                        <Text
                          as="span"
                          color="gray.400"
                          fontWeight="normal"
                          ml={2}
                        >
                          ({ext})
                        </Text>
                      </Text>
                      <Text fontSize="xs" color="gray.400" mt={0.5}>
                        {t(descriptionKey)}
                      </Text>
                    </Box>
                  </HStack>
                </Card>
              )
            )}
          </VStack>
        </ModalBody>
        <ModalFooter>
          <Button variant="ghost" onClick={modalProps.onClose}>
            {t("General.cancel")}
          </Button>
          <Button colorScheme={primaryColor} onClick={() => setActiveStep(1)}>
            {t("General.next")}
          </Button>
        </ModalFooter>
      </>
    ),
    [selectedFormat, primaryColor, modalProps.onClose, setActiveStep, t]
  );

  // ─── Step 2: Meta information ────────────────────────────────────────────────
  const metaInfoItems = useMemo(
    () => [
      {
        title: t("ExportModpackModal.meta.name"),
        children: (
          <Editable
            isTextArea={false}
            value={metaName}
            onEditSubmit={setMetaName}
            textProps={{ className: "secondary-text", fontSize: "xs-sm" }}
            inputProps={{ fontSize: "xs-sm" }}
          />
        ),
      },
      {
        title: t("ExportModpackModal.meta.author"),
        children: (
          <Editable
            isTextArea={false}
            value={metaAuthor}
            onEditSubmit={setMetaAuthor}
            textProps={{ className: "secondary-text", fontSize: "xs-sm" }}
            inputProps={{ fontSize: "xs-sm" }}
          />
        ),
      },
      {
        title: t("ExportModpackModal.meta.version"),
        children: (
          <Editable
            isTextArea={false}
            value={metaVersion}
            onEditSubmit={setMetaVersion}
            textProps={{ className: "secondary-text", fontSize: "xs-sm" }}
            inputProps={{ fontSize: "xs-sm" }}
          />
        ),
      },
      {
        title: t("ExportModpackModal.meta.description"),
        children: (
          <Editable
            isTextArea
            value={metaDescription}
            onEditSubmit={setMetaDescription}
            textProps={{ className: "secondary-text", fontSize: "xs-sm" }}
            inputProps={{ fontSize: "xs-sm" }}
          />
        ),
      },
    ],
    [metaName, metaAuthor, metaVersion, metaDescription, t]
  );

  const step2Content = useMemo(
    () => (
      <>
        <ModalBody>
          <VStack spacing={4} pt={2}>
            <OptionItemGroup
              title={t("ExportModpackModal.meta.sectionTitle")}
              items={metaInfoItems}
              w="100%"
            />
          </VStack>
        </ModalBody>
        <ModalFooter>
          <Button variant="ghost" onClick={modalProps.onClose}>
            {t("General.cancel")}
          </Button>
          <Button variant="ghost" onClick={() => setActiveStep(0)}>
            {t("General.previous")}
          </Button>
          <Button
            colorScheme={primaryColor}
            isDisabled={
              !metaName.trim() || !metaAuthor.trim() || !metaVersion.trim()
            }
            onClick={() => setActiveStep(2)}
          >
            {t("General.next")}
          </Button>
        </ModalFooter>
      </>
    ),
    [
      metaInfoItems,
      metaName,
      metaAuthor,
      metaVersion,
      primaryColor,
      modalProps.onClose,
      setActiveStep,
      t,
    ]
  );

  // ─── Step 3: File selection ──────────────────────────────────────────────────

  const progressPercent = useMemo(() => {
    if (!exportProgress || exportProgress.total === 0) return undefined;
    return Math.round((exportProgress.current / exportProgress.total) * 100);
  }, [exportProgress]);

  const progressLabel = useMemo(() => {
    if (!exportProgress) return "";
    const stageKey =
      `ExportModpackModal.progress.stage.${exportProgress.stage}` as const;
    const stageText = t(stageKey);
    if (
      exportProgress.stage === "writingManifest" ||
      exportProgress.stage === "done"
    ) {
      return stageText;
    }
    if (exportProgress.total > 0) {
      return `${stageText} (${exportProgress.current}/${exportProgress.total}) - ${exportProgress.fileName}`;
    }
    return stageText;
  }, [exportProgress, t]);

  const step3Content = useMemo(
    () => (
      <>
        <ModalBody>
          {isExporting ? (
            <VStack spacing={4} py={8} px={2}>
              <Text
                fontSize="sm"
                color="gray.400"
                noOfLines={1}
                w="100%"
                textAlign="center"
              >
                {progressLabel}
              </Text>
              <Progress
                w="100%"
                size="sm"
                borderRadius="full"
                colorScheme={primaryColor}
                value={progressPercent}
                isIndeterminate={progressPercent === undefined}
                hasStripe
                isAnimated
              />
              {progressPercent !== undefined && (
                <Text fontSize="xs" color="gray.500">
                  {progressPercent}%
                </Text>
              )}
            </VStack>
          ) : isLoadingFiles ? (
            <Center py={8}>
              <BeatLoader size={14} color="gray" />
            </Center>
          ) : (
            <VStack spacing={4} w="100%">
              <FileTreeSelector
                entries={fileEntries}
                selected={selectedFiles}
                onChange={setSelectedFiles}
              />
              <OptionItemGroup
                title={t("ExportModpackModal.options.sectionTitle")}
                items={
                  [
                    ...(selectedFormat === ExportFormat.Modrinth
                      ? [
                          {
                            title: t(
                              "ExportModpackModal.options.noCreateRemoteFiles"
                            ),
                            children: (
                              <Switch
                                colorScheme={primaryColor}
                                isChecked={noCreateRemoteFiles}
                                onChange={(e) =>
                                  setNoCreateRemoteFiles(e.target.checked)
                                }
                              />
                            ),
                          },
                          {
                            title: t(
                              "ExportModpackModal.options.skipCurseForgeRemoteFiles"
                            ),
                            children: (
                              <Switch
                                colorScheme={primaryColor}
                                isChecked={skipCurseForgeRemoteFiles}
                                onChange={(e) =>
                                  setSkipCurseForgeRemoteFiles(e.target.checked)
                                }
                              />
                            ),
                          },
                        ]
                      : []),
                    ...(selectedFormat === ExportFormat.MultiMC
                      ? [
                          {
                            title: t("ExportModpackModal.options.minMemory"),
                            children: (
                              <HStack spacing={2}>
                                <NumberInput
                                  min={256}
                                  size="xs"
                                  maxW={24}
                                  clampValueOnBlur={false}
                                  focusBorderColor={`${primaryColor}.500`}
                                  value={minMemory}
                                  onChange={(value) => {
                                    if (/^\d*$/.test(value))
                                      setMinMemory(value);
                                  }}
                                >
                                  <NumberInputField pr={0} fontSize="xs-sm" />
                                </NumberInput>
                                <Text fontSize="xs" color="gray.500">
                                  MB
                                </Text>
                              </HStack>
                            ),
                          },
                        ]
                      : []),
                  ].filter(Boolean) as any
                }
                w="100%"
              />
            </VStack>
          )}
        </ModalBody>
        <ModalFooter>
          <Button
            variant="ghost"
            onClick={modalProps.onClose}
            isDisabled={isExporting}
          >
            {t("General.cancel")}
          </Button>
          <Button
            variant="ghost"
            onClick={() => setActiveStep(1)}
            isDisabled={isExporting}
          >
            {t("General.previous")}
          </Button>
          <Button
            colorScheme={primaryColor}
            isLoading={isExporting}
            isDisabled={selectedFiles.size === 0}
            onClick={handleExport}
          >
            {t("ExportModpackModal.button.export")}
          </Button>
        </ModalFooter>
      </>
    ),
    [
      isLoadingFiles,
      fileEntries,
      selectedFiles,
      isExporting,
      progressPercent,
      progressLabel,
      handleExport,
      primaryColor,
      modalProps.onClose,
      setActiveStep,
      t,
      selectedFormat,
      noCreateRemoteFiles,
      skipCurseForgeRemoteFiles,
      minMemory,
    ]
  );

  const steps = useMemo(
    () => [
      {
        key: "format",
        content: step1Content,
        description: selectedFormat,
      },
      {
        key: "meta",
        content: step2Content,
        description: metaName,
      },
      {
        key: "files",
        content: step3Content,
        description: "",
      },
    ],
    [step1Content, step2Content, step3Content, selectedFormat, metaName]
  );

  return (
    <Modal
      scrollBehavior="inside"
      size={{ base: "2xl", lg: "3xl", xl: "4xl" }}
      autoFocus={false}
      {...modalProps}
    >
      <ModalOverlay />
      <ModalContent h="100%">
        <ModalHeader>{t("ExportModpackModal.header.title")}</ModalHeader>
        <ModalCloseButton />
        <Center>
          <Stepper
            colorScheme={primaryColor}
            index={activeStep}
            w="80%"
            my={1.5}
          >
            {steps.map((step, index) => (
              <Step key={index}>
                <StepIndicator>
                  <StepStatus
                    complete={<StepIcon />}
                    incomplete={<StepNumber />}
                    active={<StepNumber />}
                  />
                </StepIndicator>
                <Box flexShrink="0">
                  <StepTitle fontSize="sm">
                    {t(`ExportModpackModal.stepper.${step.key}`)}
                  </StepTitle>
                </Box>
                <StepSeparator />
              </Step>
            ))}
          </Stepper>
        </Center>
        <Flex flexGrow="1" flexDir="column" h="100%" overflow="auto">
          {steps[activeStep].content}
        </Flex>
      </ModalContent>
    </Modal>
  );
};

export default ExportModpackModal;
