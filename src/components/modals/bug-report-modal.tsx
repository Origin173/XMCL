import {
  Button,
  Checkbox,
  FormControl,
  FormLabel,
  HStack,
  Input,
  Link,
  Modal,
  ModalBody,
  ModalCloseButton,
  ModalContent,
  ModalFooter,
  ModalHeader,
  ModalOverlay,
  ModalProps,
  Text,
  Textarea,
  VStack,
} from "@chakra-ui/react";
import { save } from "@tauri-apps/plugin-dialog";
import { openUrl, revealItemInDir } from "@tauri-apps/plugin-opener";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import { LuCopy, LuExternalLink, LuFileArchive } from "react-icons/lu";
import { useLauncherConfig } from "@/contexts/config";
import { useToast } from "@/contexts/toast";
import { UtilsService } from "@/services/utils";
import { copyText } from "@/utils/copy";

const ISSUE_URL = "https://github.com/Origin173/XMCL/issues/new";
const QQ_GROUP = "496981669";
// keep the prefilled issue URL well under the length browsers and GitHub accept
const MAX_FIELD_LENGTH = 1500;

const BugReportModal: React.FC<Omit<ModalProps, "children">> = (modalProps) => {
  const { t, i18n } = useTranslation();
  const toast = useToast();
  const { config } = useLauncherConfig();
  const primaryColor = config.appearance.theme.primaryColor;
  const basicInfo = config.basicInfo;

  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");
  const [steps, setSteps] = useState("");
  const [expected, setExpected] = useState("");
  const [includeLogs, setIncludeLogs] = useState(true);
  const [isExporting, setIsExporting] = useState(false);
  const [exportedPath, setExportedPath] = useState<string>();

  const osText = `${basicInfo.osType} ${basicInfo.platformVersion} (${basicInfo.arch})`;
  const isDescriptionEmpty = !description.trim();

  const buildPlainText = () =>
    [
      `[Bug] ${title.trim()}`,
      `${t("BugReportModal.field.description")}: ${description.trim()}`,
      steps.trim() && `${t("BugReportModal.field.steps")}: ${steps.trim()}`,
      expected.trim() &&
        `${t("BugReportModal.field.expected")}: ${expected.trim()}`,
      `XMCL ${basicInfo.launcherVersion} / ${osText}`,
    ]
      .filter(Boolean)
      .join("\n");

  const handleExport = async () => {
    const timestamp = new Date()
      .toISOString()
      .slice(0, 19)
      .replace(/[:T]/g, "-");
    const savePath = await save({
      defaultPath: `xmcl-diagnostic-${timestamp}.zip`,
      filters: [{ name: "ZIP", extensions: ["zip"] }],
    });
    if (!savePath) return;

    setIsExporting(true);
    const response = await UtilsService.exportDiagnosticReport(
      buildPlainText(),
      includeLogs,
      savePath
    );
    setIsExporting(false);

    if (response.status === "success") {
      setExportedPath(response.data);
      toast({ title: response.message, status: "success" });
      revealItemInDir(response.data);
    } else {
      toast({
        title: response.message,
        description: response.details,
        status: "error",
      });
    }
  };

  const handleOpenIssue = () => {
    const params = new URLSearchParams({
      template: i18n.language.startsWith("zh") ? "zh.bug.yml" : "bug.yml",
      title: `[Bug] ${title.trim()}`,
      description: description.trim().slice(0, MAX_FIELD_LENGTH),
      steps: steps.trim().slice(0, MAX_FIELD_LENGTH),
      expected: expected.trim().slice(0, MAX_FIELD_LENGTH),
      os: osText,
      launcherVersion: basicInfo.launcherVersion,
      context: exportedPath ? t("BugReportModal.issue.attachHint") : "",
    });
    openUrl(`${ISSUE_URL}?${params.toString()}`);
  };

  return (
    <Modal size={{ base: "lg", lg: "xl" }} {...modalProps}>
      <ModalOverlay />
      <ModalContent>
        <ModalHeader>{t("BugReportModal.header")}</ModalHeader>
        <ModalCloseButton />
        <ModalBody>
          <VStack align="stretch" spacing={3}>
            <FormControl>
              <FormLabel fontSize="sm">
                {t("BugReportModal.field.title")}
              </FormLabel>
              <Input
                size="sm"
                value={title}
                onChange={(e) => setTitle(e.target.value)}
                placeholder={t("BugReportModal.placeholder.title")}
                focusBorderColor={`${primaryColor}.500`}
              />
            </FormControl>
            <FormControl isRequired>
              <FormLabel fontSize="sm">
                {t("BugReportModal.field.description")}
              </FormLabel>
              <Textarea
                size="sm"
                rows={3}
                value={description}
                onChange={(e) => setDescription(e.target.value)}
                placeholder={t("BugReportModal.placeholder.description")}
                focusBorderColor={`${primaryColor}.500`}
              />
            </FormControl>
            <FormControl>
              <FormLabel fontSize="sm">
                {t("BugReportModal.field.steps")}
              </FormLabel>
              <Textarea
                size="sm"
                rows={2}
                value={steps}
                onChange={(e) => setSteps(e.target.value)}
                placeholder={t("BugReportModal.placeholder.steps")}
                focusBorderColor={`${primaryColor}.500`}
              />
            </FormControl>
            <FormControl>
              <FormLabel fontSize="sm">
                {t("BugReportModal.field.expected")}
              </FormLabel>
              <Input
                size="sm"
                value={expected}
                onChange={(e) => setExpected(e.target.value)}
                focusBorderColor={`${primaryColor}.500`}
              />
            </FormControl>
            <Checkbox
              colorScheme={primaryColor}
              isChecked={includeLogs}
              onChange={(e) => setIncludeLogs(e.target.checked)}
            >
              <Text fontSize="sm">{t("BugReportModal.includeLogs")}</Text>
            </Checkbox>
            <Text fontSize="xs" className="secondary-text">
              {t("BugReportModal.privacyNote")}
            </Text>
            <HStack fontSize="sm" spacing={1} wrap="wrap">
              <Text>{t("BugReportModal.qqGroup", { group: QQ_GROUP })}</Text>
              <Link
                color={`${primaryColor}.500`}
                onClick={() => copyText(QQ_GROUP, { toast })}
              >
                {t("BugReportModal.button.copyGroup")}
              </Link>
              <Text>·</Text>
              <Link
                color={`${primaryColor}.500`}
                onClick={() =>
                  openUrl(
                    `${ISSUE_URL}?template=${i18n.language.startsWith("zh") ? "zh.feature.yml" : "feature.yml"}`
                  )
                }
              >
                {t("BugReportModal.button.featureRequest")}
              </Link>
            </HStack>
          </VStack>
        </ModalBody>
        <ModalFooter>
          <HStack spacing={2} w="100%" justify="flex-end" wrap="wrap">
            <Button
              size="sm"
              variant="ghost"
              leftIcon={<LuCopy />}
              isDisabled={isDescriptionEmpty}
              onClick={() => copyText(buildPlainText(), { toast })}
            >
              {t("BugReportModal.button.copyText")}
            </Button>
            <Button
              size="sm"
              variant="outline"
              colorScheme={primaryColor}
              leftIcon={<LuFileArchive />}
              isDisabled={isDescriptionEmpty}
              isLoading={isExporting}
              onClick={handleExport}
            >
              {t("BugReportModal.button.export")}
            </Button>
            <Button
              size="sm"
              colorScheme={primaryColor}
              leftIcon={<LuExternalLink />}
              isDisabled={isDescriptionEmpty || !title.trim()}
              onClick={handleOpenIssue}
            >
              {t("BugReportModal.button.openIssue")}
            </Button>
          </HStack>
        </ModalFooter>
      </ModalContent>
    </Modal>
  );
};

export default BugReportModal;
