import {
  Button,
  FormControl,
  FormErrorMessage,
  FormLabel,
  Input,
  Modal,
  ModalBody,
  ModalCloseButton,
  ModalContent,
  ModalFooter,
  ModalHeader,
  ModalOverlay,
  Text,
  VStack,
  useDisclosure,
} from "@chakra-ui/react";
import React, { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { useToast } from "@/contexts/toast";
import { CAUCService } from "@/services/cauc";

interface CAUCLoginModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSuccess?: () => void;
}

/**
 * CAUC (中国民航大学) 登录模态框组件。
 *
 * 认证流程:
 * 1. 使用学工号 + OA 密码完成 CAUC 校园统一身份登录
 * 2. 根据返回决定是否需要绑定昵称
 * 3. 完成 Yggdrasil 认证并写入启动器账号列表
 */
export const CAUCLoginModal: React.FC<CAUCLoginModalProps> = ({
  isOpen,
  onClose,
  onSuccess,
}) => {
  const { t } = useTranslation();
  const toast = useToast();

  // 表单状态
  const [studentId, setStudentId] = useState("");
  const [oaPassword, setOaPassword] = useState("");
  const [playerName, setPlayerName] = useState("");
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [requiresBind, setRequiresBind] = useState(false);
  const [savedCredentials, setSavedCredentials] = useState<{
    studentId: string;
    oaPassword: string;
  } | null>(null);
  const credentials = useMemo(
    () => ({ studentId, oaPassword }),
    [studentId, oaPassword]
  );

  // 表单验证错误
  const [studentIdError, setStudentIdError] = useState("");
  const [passwordError, setPasswordError] = useState("");
  const [playerNameError, setPlayerNameError] = useState("");

  const resetStates = () => {
    setStudentId("");
    setOaPassword("");
    setPlayerName("");
    setStudentIdError("");
    setPasswordError("");
    setPlayerNameError("");
    setRequiresBind(false);
    setIsSubmitting(false);
    setSavedCredentials(null);
  };

  const handleClose = () => {
    resetStates();
    onClose();
  };

  const validateInputs = (): boolean => {
    let isValid = true;

    if (!studentId.trim()) {
      setStudentIdError("请输入学工号");
      isValid = false;
    } else {
      setStudentIdError("");
    }

    if (!oaPassword) {
      setPasswordError("请输入密码");
      isValid = false;
    } else {
      setPasswordError("");
    }

    if (requiresBind) {
      if (!playerName.trim()) {
        setPlayerNameError("请输入游戏昵称");
        isValid = false;
      } else {
        setPlayerNameError("");
      }
    }

    return isValid;
  };

  const handleCaucLogin = async () => {
    if (!validateInputs()) {
      return;
    }

    setIsSubmitting(true);

    try {
      console.log("Attempting CAUC login with student ID:", studentId);

      const response = await CAUCService.caucLogin(studentId, oaPassword);

      console.log("CAUC login response:", response);

      if (response.status === "success") {
        const needBind = Boolean(response.data);
        setRequiresBind(needBind);
        setSavedCredentials({
          studentId,
          oaPassword,
        });

        if (needBind) {
          toast({
            title: "登录成功",
            description: "检测到首次使用，请设置游戏昵称",
            status: "info",
          });
        } else {
          await completeLogin(studentId, oaPassword);
        }
      } else {
        console.error("CAUC login failed:", response);
        toast({
          title: response.message || "登录失败",
          description: response.details || "学工号或密码错误",
          status: "error",
          duration: 5000,
        });
      }
    } catch (error) {
      console.error("CAUC login error:", error);
      toast({
        title: "登录失败",
        description:
          error instanceof Error ? error.message : "网络错误或服务器异常",
        status: "error",
        duration: 5000,
      });
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleBindPlayerName = async () => {
    if (!validateInputs()) {
      return;
    }

    setIsSubmitting(true);

    try {
      const response = await CAUCService.caucBindPlayerName(playerName);

      if (response.status === "success") {
        toast({
          title: "昵称绑定成功",
          description: `游戏昵称 "${playerName}" 已绑定`,
          status: "success",
        });

        const creds = savedCredentials ?? credentials;
        await completeLogin(creds.studentId, creds.oaPassword);
      } else {
        toast({
          title: response.message || "绑定失败",
          description: response.details || "昵称可能已被占用",
          status: "error",
          duration: 5000,
        });
      }
    } catch (error) {
      toast({
        title: "绑定失败",
        description:
          error instanceof Error ? error.message : "网络错误或服务器异常",
        status: "error",
        duration: 5000,
      });
    } finally {
      setIsSubmitting(false);
    }
  };

  const completeLogin = async (id: string, password: string) => {
    setIsSubmitting(true);

    try {
      const response = await CAUCService.caucCompleteLogin(id, password);

      console.log("CAUC complete login response:", response);

      if (response.status === "success") {
        if (response.data && response.data.length > 0) {
          toast({
            title: "需要选择角色",
            description: "检测到多个角色，请在角色列表中选择一个",
            status: "info",
          });
        } else {
          toast({
            title: "登录成功",
            description: "CAUC 账号已添加到启动器",
            status: "success",
          });
        }

        resetStates();
        onSuccess?.();
        onClose();
      } else {
        toast({
          title: response.message || "登录失败",
          description: response.details || "无法完成认证",
          status: "error",
          duration: 5000,
        });
      }
    } catch (error) {
      console.error("CAUC complete login error:", error);
      toast({
        title: "登录失败",
        description:
          error instanceof Error ? error.message : "网络错误或服务器异常",
        status: "error",
        duration: 5000,
      });
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <Modal isOpen={isOpen} onClose={handleClose} size="md" isCentered>
      <ModalOverlay />
      <ModalContent>
        <ModalHeader>
          {requiresBind ? "设置游戏昵称" : "中国民航大学账号登录"}
        </ModalHeader>
        <ModalCloseButton />

        <ModalBody>
          {!requiresBind ? (
            <VStack spacing={4} align="stretch">
              <Text fontSize="sm" color="gray.500">
                请使用您的学工号和 OA 密码登录
              </Text>

              <FormControl isInvalid={!!studentIdError}>
                <FormLabel>学工号</FormLabel>
                <Input
                  placeholder="请输入学工号"
                  value={studentId}
                  onChange={(e) => setStudentId(e.target.value)}
                  onKeyPress={(e) => e.key === "Enter" && handleCaucLogin()}
                />
                <FormErrorMessage>{studentIdError}</FormErrorMessage>
              </FormControl>

              <FormControl isInvalid={!!passwordError}>
                <FormLabel>OA 密码</FormLabel>
                <Input
                  type="password"
                  placeholder="请输入 OA 密码"
                  value={oaPassword}
                  onChange={(e) => setOaPassword(e.target.value)}
                  onKeyPress={(e) => e.key === "Enter" && handleCaucLogin()}
                />
                <FormErrorMessage>{passwordError}</FormErrorMessage>
              </FormControl>

              <Text fontSize="xs" color="gray.400">
                首次登录将自动注册 Minecraft 账号
              </Text>
            </VStack>
          ) : (
            <VStack spacing={4} align="stretch">
              <Text fontSize="sm" color="gray.500">
                检测到您是新用户，请设置您的 Minecraft 游戏昵称
              </Text>

              <FormControl isInvalid={!!playerNameError}>
                <FormLabel>游戏昵称</FormLabel>
                <Input
                  placeholder="请输入游戏昵称"
                  value={playerName}
                  onChange={(e) => setPlayerName(e.target.value)}
                  onKeyPress={(e) =>
                    e.key === "Enter" && handleBindPlayerName()
                  }
                />
                <FormErrorMessage>{playerNameError}</FormErrorMessage>
              </FormControl>

              <Text fontSize="xs" color="gray.400">
                昵称设置后将无法修改，请谨慎选择
              </Text>
            </VStack>
          )}
        </ModalBody>

        <ModalFooter>
          <Button variant="ghost" mr={3} onClick={handleClose}>
            取消
          </Button>
          <Button
            colorScheme="blue"
            onClick={requiresBind ? handleBindPlayerName : handleCaucLogin}
            isLoading={isSubmitting}
            loadingText={requiresBind ? "处理中..." : "登录中..."}
          >
            {requiresBind ? "确认绑定" : "登录"}
          </Button>
        </ModalFooter>
      </ModalContent>
    </Modal>
  );
};

/**
 * 用于快速触发 CAUC 登录的 Hook
 */
export const useCAUCLogin = () => {
  const { isOpen, onOpen, onClose } = useDisclosure();

  return {
    isOpen,
    openCAUCLogin: onOpen,
    closeCAUCLogin: onClose,
  };
};
