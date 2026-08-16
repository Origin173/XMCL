// This file intentionally left minimal; the real implementation lives in
// `cauc-login-modal-simple.tsx`. Keeping the placeholder allows downstream
// imports that still point to this path to compile without bringing in unused
// UI code during the build.
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
import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import { useToast } from "@/contexts/toast";
import { CAUCService } from "@/services/cauc";

interface CAUCLoginModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSuccess?: () => void;
}

const CAUC_AUTH_SERVER = "https://skin.cauc.fun/api/yggdrasil";

/**
 * CAUC (中国民航大学) 登录模态框组件
 *
 * 使用标准的第三方认证服务器登录流程
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
  const [isLoading, setIsLoading] = useState(false);
  const [requiresBind, setRequiresBind] = useState(false);
  const [credentialsSaved, setCredentialsSaved] = useState<{
    studentId: string;
    oaPassword: string;
  } | null>(null);

  // 错误状态
  const [studentIdError, setStudentIdError] = useState("");
  const [passwordError, setPasswordError] = useState("");
  const [playerNameError, setPlayerNameError] = useState("");

  // 重置所有状态
  const resetStates = () => {
    setStudentId("");
    setOaPassword("");
    setPlayerName("");
    setRequiresBind(false);
    setCredentialsSaved(null);
    setStudentIdError("");
    setPasswordError("");
    setPlayerNameError("");
  };

  // 处理关闭
  const handleClose = () => {
    resetStates();
    onClose();
  };

  // 验证输入
  const validateInputs = () => {
    let isValid = true;

    if (!studentId.trim()) {
      setStudentIdError("请输入学工号");
      isValid = false;
    } else {
      setStudentIdError("");
    }

    if (!oaPassword.trim()) {
      setPasswordError("请输入 OA 密码");
      isValid = false;
    } else {
      setPasswordError("");
    }

    if (requiresBind && !playerName.trim()) {
      setPlayerNameError("请输入游戏昵称");
      isValid = false;
    } else {
      setPlayerNameError("");
    }

    return isValid;
  };

  // 步骤 1: 登录
  const handleLogin = async () => {
    if (!validateInputs()) return;

    setIsLoading(true);

    try {
      const response = await CAUCService.caucLogin(studentId, oaPassword);

      console.log("CAUC login response:", response);

      if (response.status === "success") {
        // 保存凭据供后续使用
        setCredentialsSaved({ studentId, oaPassword });

        if (response.data) {
          // 需要绑定昵称
          setRequiresBind(true);
          toast({
            title: "登录成功",
            description: "检测到您是新用户,请设置游戏昵称",
            status: "info",
          });
        } else {
          // 直接完成登录
          await completeLogin(studentId, oaPassword);
        }
      } else {
        console.error("CAUC login failed:", response);
        toast({
          title: "登录失败",
          description: response.message || "学工号或密码错误",
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
      setIsLoading(false);
    }
  };

  // 步骤 2: 绑定昵称
  const handleBindPlayerName = async () => {
    if (!validateInputs()) return;

    setIsLoading(true);

    try {
      const bindResponse = await CAUCService.caucBindPlayerName(playerName);

      if (bindResponse.status === "success") {
        toast({
          title: "昵称绑定成功",
          description: `游戏昵称 "${playerName}" 已绑定`,
          status: "success",
        });

        // 完成登录
        if (credentialsSaved) {
          await completeLogin(
            credentialsSaved.studentId,
            credentialsSaved.oaPassword
          );
        }
      } else {
        toast({
          title: "绑定失败",
          description: bindResponse.message || "昵称可能已被占用",
          status: "error",
        });
      }
    } catch (error) {
      toast({
        title: "绑定失败",
        description: "网络错误或昵称不可用",
        status: "error",
      });
    } finally {
      setIsLoading(false);
    }
  };

  // 步骤 3: 完成登录
  const completeLogin = async (studentId: string, oaPassword: string) => {
    console.log("Starting completeLogin with studentId:", studentId);

    setIsLoading(true);

    try {
      const response = await CAUCService.caucCompleteLogin(
        studentId,
        oaPassword
      );

      console.log("CAUC complete login response:", response);

      if (response.status === "success") {
        if (response.data && response.data.length > 0) {
          // 有多个角色需要选择,显示选择界面
          console.log(
            "Multiple players available, need selection:",
            response.data
          );
          toast({
            title: "需要选择角色",
            description: "检测到多个游戏角色,请选择一个",
            status: "info",
          });
          // TODO: 实现角色选择界面
        } else {
          // 成功添加账号
          toast({
            title: "登录成功",
            description: "CAUC 账号已添加到启动器",
            status: "success",
          });

          resetStates();
          onSuccess?.();
          onClose();
        }
      } else {
        console.error("CAUC complete login failed:", response);
        toast({
          title: "登录失败",
          description: response.message || "无法完成认证",
          status: "error",
          duration: 5000,
        });
      }
    } catch (error) {
      console.error("CAUC complete login error:", error);
      toast({
        title: "登录失败",
        description:
          error instanceof Error ? error.message : "完成认证时发生错误",
        status: "error",
        duration: 5000,
      });
    } finally {
      setIsLoading(false);
    }
  };

  // 绑定昵称需要的额外状态
  const [playerName, setPlayerName] = useState("");

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
            // 登录界面
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
                  onKeyPress={(e) => e.key === "Enter" && handleLogin()}
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
                  onKeyPress={(e) => e.key === "Enter" && handleLogin()}
                />
                <FormErrorMessage>{passwordError}</FormErrorMessage>
              </FormControl>

              <Text fontSize="xs" color="gray.400">
                首次登录将自动注册 Minecraft 账号
              </Text>
            </VStack>
          ) : (
            // 昵称绑定界面
            <VStack spacing={4} align="stretch">
              <Text fontSize="sm" color="gray.500">
                检测到您是新用户,请设置您的 Minecraft 游戏昵称
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
                昵称一旦设置将无法修改,请谨慎选择
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
            onClick={requiresBind ? handleBindPlayerName : handleLogin}
            isLoading={isLoading}
            loadingText={requiresBind ? "绑定中..." : "登录中..."}
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
