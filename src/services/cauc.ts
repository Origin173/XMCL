import { invoke } from "@tauri-apps/api/core";
import { Player } from "@/models/account";
import { InvokeResponse } from "@/models/response";
import { responseHandler } from "@/utils/response";

/**
 * CAUC (中国民航大学) 认证服务
 *
 * 认证流程:
 * 1. caucLogin - 使用学工号和OA密码登录
 * 2. 如果返回 true,需要绑定昵称,调用 caucBindPlayerName
 * 3. caucCompleteLogin - 完成认证并添加玩家到账号列表
 */
export class CAUCService {
  /**
   * 步骤 1: 使用学工号和 OA 密码提交 CAUC 表单登录
   *
   * @param {string} studentId - 学工号
   * @param {string} oaPassword - OA 密码
   * @returns {Promise<InvokeResponse<boolean>>} true 表示需要绑定昵称,false 表示可以直接登录
   */
  @responseHandler("cauc")
  static async caucLogin(
    studentId: string,
    oaPassword: string
  ): Promise<InvokeResponse<boolean>> {
    return await invoke("cauc_login", {
      studentId,
      oaPassword,
    });
  }

  /**
   * 步骤 2: 绑定游戏昵称 (仅新用户需要)
   *
   * 这个方法必须在 caucLogin 返回 true 后调用
   *
   * @param {string} playerName - 游戏昵称
   * @returns {Promise<InvokeResponse<void>>}
   */
  @responseHandler("cauc")
  static async caucBindPlayerName(
    playerName: string
  ): Promise<InvokeResponse<void>> {
    return await invoke("cauc_bind_player_name", {
      playerName,
    });
  }

  /**
   * 步骤 3: 完成认证并添加玩家到账号列表
   *
   * 这个方法在绑定昵称后(或不需要绑定时)调用
   *
   * @param {string} studentId - 学工号
   * @param {string} oaPassword - OA 密码
   * @returns {Promise<InvokeResponse<Player[]>>} 空数组表示已添加,非空数组表示需要用户选择
   */
  @responseHandler("cauc")
  static async caucCompleteLogin(
    studentId: string,
    oaPassword: string
  ): Promise<InvokeResponse<Player[]>> {
    return await invoke("cauc_complete_login", {
      studentId,
      oaPassword,
    });
  }
}
