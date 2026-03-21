
/* WARNING: Type propagation algorithm not settling */
/* WARNING: Globals starting with '_' overlap smaller symbols at the same address */

void GameServerReceiver$$OnReceive
               (GameServerReceiver_o *__this,Packet_o *incoming,MethodInfo *method)

{
  bool bVar1;
  uint uVar2;
  UnityEngine_Rect_o rect;
  _union_13 _Var3;
  char *pcVar4;
  bool bVar5;
  uint8_t uVar7;
  uint8_t uVar8;
  bool bVar9;
  int16_t iVar10;
  int16_t iVar11;
  int16_t iVar12;
  int16_t iVar13;
  int16_t iVar14;
  int16_t iVar15;
  int16_t iVar16;
  int32_t iVar17;
  uint uVar18;
  int iVar19;
  int iVar20;
  int32_t iVar21;
  int32_t iVar22;
  int32_t iVar23;
  uint8_t uVar6;
  System_String_o *pSVar24;
  Connection_o *pCVar25;
  SharedCreature_o *pSVar26;
  System_Collections_Generic_List_string__o *pSVar27;
  System_Collections_Generic_List_GameObject__o *__this_00;
  PerkData_o *pPVar28;
  System_String_o *pSVar29;
  PerkControl_o *pPVar30;
  System_Collections_Generic_Dictionary_string__BanditCampInstance__o *pSVar31;
  ulong uVar32;
  BanditCampInstance_o *pBVar33;
  Packet_o *pPVar34;
  GameServerReceiver_o *extraout_x0;
  GameServerSender_o *__this_01;
  GameServerReceiver_o *extraout_x0_00;
  GameServerReceiver_o *extraout_x0_01;
  System_Collections_Generic_Dictionary_string__CreatureStruct__o *pSVar35;
  OnlineTeleporter_o **ppOVar36;
  UnityEngine_Component_o *pUVar37;
  MusicBoxControl_o *pMVar38;
  OnlinePlayerData_o *__this_02;
  System_Collections_Generic_Dictionary_string__OnlinePlayer__o *pSVar39;
  OnlinePlayer_o *player;
  GameServerReceiver_o *__this_03;
  ChunkElement_o *__this_04;
  GameServerReceiver_o *pGVar40;
  BasketContents_o *pBVar41;
  long lVar42;
  LandClaimChunkTimer_o *pLVar43;
  System_Collections_Generic_Dictionary_string__LandClaimChunkTimer__o *pSVar44;
  System_String_o *str2;
  GameServerReceiver_o *extraout_x0_02;
  GameServerReceiver_o *extraout_x0_03;
  GameServerReceiver_o *__this_05;
  GameServerReceiver_o *__this_06;
  ChunkData_o *pCVar45;
  System_Collections_Generic_Dictionary_string__ChunkData__o *pSVar46;
  GameServerReceiver_o *__this_07;
  Chunk_o *pCVar47;
  UnityEngine_Texture2D_o *__this_08;
  UnityEngine_Sprite_o *pUVar48;
  CompanionController_o *pCVar49;
  System_Byte_array *pSVar50;
  PoolGameRecording_o *__this_09;
  System_Collections_Generic_List_ActiveCompanion__o *__this_10;
  WindowControl_o *pWVar51;
  PopupControl_o *pPVar52;
  System_Collections_Generic_List_Friend__o *__this_11;
  _union_13 _Var53;
  Il2CppClass *pIVar54;
  System_Collections_Generic_Dictionary_string__GameObject__o *pSVar55;
  UnityEngine_GameObject_o *pUVar56;
  Il2CppObject *pIVar57;
  System_Collections_Generic_List_object__o *pSVar58;
  UnityEngine_Transform_o *pUVar59;
  CreatureStruct_o *pCVar60;
  long lVar61;
  Combatant_o *pCVar62;
  System_String_o *pSVar63;
  PlayerData_o *pPVar64;
  System_String_o *pSVar65;
  PerkReceiver_o *pPVar66;
  LandClaimControl_o *pLVar67;
  InventoryItem_o *pIVar68;
  InventoryItem_o *pIVar69;
  InventoryItem_o *pIVar70;
  InventoryItem_o **ppIVar71;
  System_Collections_Generic_List_int__o *pSVar72;
  _union_13 *p_Var73;
  UnityEngine_Object_o *pUVar74;
  System_Int32_array *pSVar75;
  GameplayGUIControl_o *pGVar76;
  GameServerReceiver_o *__this_12;
  GameServerSender_o *extraout_x0_04;
  GameServerReceiver_o *__this_13;
  ZoneData_o *pZVar77;
  GameServerSender_o *extraout_x0_05;
  GameServerSender_o *extraout_x0_06;
  QuestControl_o *pQVar78;
  System_Collections_Generic_List_CraftingSlot__o *pSVar79;
  chat_log_o *pcVar80;
  ChatCollection_o *pCVar81;
  inventory_ctr_c *piVar82;
  FriendServerSender_o *pFVar83;
  System_DateTime_o SVar84;
  ChunkData_o **ppCVar85;
  UnityEngine_GameObject_o *pUVar86;
  GameServerReceiver_o *__this_14;
  GameServerReceiver_o *__this_15;
  GameServerSender_o *extraout_x0_07;
  GameServerSender_o *extraout_x0_08;
  GameServerReceiver_o *__this_16;
  System_Action_o *pSVar87;
  undefined8 uVar88;
  MethodInfo *extraout_x1;
  MethodInfo *extraout_x1_00;
  MethodInfo *extraout_x1_01;
  MethodInfo *method_00;
  MethodInfo *extraout_x1_02;
  MethodInfo *extraout_x1_03;
  MethodInfo *extraout_x1_04;
  MethodInfo *extraout_x1_05;
  MethodInfo *extraout_x1_06;
  MethodInfo *extraout_x1_07;
  MethodInfo *extraout_x1_08;
  MethodInfo *extraout_x1_09;
  MethodInfo *extraout_x1_10;
  MethodInfo *extraout_x1_11;
  System_Collections_IEnumerator_o *routine;
  MethodInfo *extraout_x1_12;
  MethodInfo *extraout_x1_13;
  MethodInfo *extraout_x1_14;
  MethodInfo *extraout_x1_15;
  MethodInfo *extraout_x1_16;
  MethodInfo *extraout_x1_17;
  MethodInfo *extraout_x1_18;
  MethodInfo *extraout_x1_19;
  MethodInfo *extraout_x1_20;
  MethodInfo *method_01;
  MethodInfo *method_02;
  MethodInfo *extraout_x1_21;
  MethodInfo *extraout_x1_22;
  MethodInfo *extraout_x1_23;
  MethodInfo *extraout_x1_24;
  MethodInfo *extraout_x1_25;
  MethodInfo *extraout_x1_26;
  MethodInfo *extraout_x1_27;
  MethodInfo *method_03;
  Startup_c **method_04;
  MethodInfo *method_05;
  MethodInfo_F19A34 **method_06;
  inventory_ctr_c **method_07;
  MethodInfo **method_08;
  char cVar89;
  GameServerConnector_o *pGVar90;
  LiteModel_o *pLVar91;
  CreatureModel_o *pCVar92;
  GameController_o *pGVar93;
  BanditCampsControl_o *pBVar94;
  GameServerInterface_o *pGVar95;
  long lVar96;
  ulong uVar97;
  ZoneData_o **ppZVar98;
  System_Collections_Generic_Dictionary_string__Sprite__o *pSVar99;
  undefined8 *puVar100;
  FriendServerReceiver_o *pFVar101;
  MethodInfo *pMVar102;
  MobControl_o *pMVar103;
  ConstructionControl_o *pCVar104;
  CustomTeleporterControl_o *pCVar105;
  GameServerSender_o *pGVar106;
  System_Collections_Generic_Dictionary_string__List_int___o *pSVar107;
  FriendServerInterface_o *pFVar108;
  PopupControl_StaticFields *pPVar109;
  UnityEngine_Vector3_StaticFields *pUVar110;
  OnlineTeleporter_o *pOVar111;
  inventory_ctr_o *piVar112;
  UnityEngine_GameObject_o **ppUVar113;
  System_Object_array *pSVar114;
  System_String_array *pSVar115;
  PoolGameControl_o *pPVar116;
  TradingTableControl_o *pTVar117;
  System_Action_o **ppSVar118;
  Il2CppObject *pIVar119;
  LootControl_o *__this_17;
  ChunkControl_o *pCVar120;
  System_String_o **ppSVar121;
  MinigameMenu_o *pMVar122;
  ZoneDataControl_o *pZVar123;
  double dVar124;
  undefined8 in_d4;
  undefined8 in_d5;
  float in_s6;
  float in_s7;
  float fVar125;
  double dVar126;
  float fVar127;
  float fVar128;
  UnityEngine_Vector3_o UVar129;
  UnityEngine_Vector3_o UVar130;
  UnityEngine_Vector3_o pos;
  UnityEngine_Color_o creature_name_col;
  UnityEngine_Quaternion_o UVar131;
  UnityEngine_Vector2_o V;
  undefined1 auVar132 [16];
  undefined1 auStack_188 [48];
  Il2CppType **ppIStack_158;
  _union_13 local_150;
  _union_14 _Stack_148;
  System_Collections_Generic_List_T__o *local_140;
  Il2CppType **ppIStack_138;
  _union_13 _Stack_130;
  char *pcStack_128;
  Il2CppClass *local_120;
  undefined1 local_110 [8];
  Il2CppType **local_108;
  _union_13 _Stack_100;
  undefined1 local_f0 [8];
  Il2CppType **ppIStack_e8;
  _union_13 local_e0;
  undefined8 uStack_d8;
  undefined8 local_d0;
  System_DateTime_Fields SStack_c8;
  int local_bc;
  undefined8 local_b8;
  undefined8 uStack_b0;
  UnityEngine_SceneManagement_Scene_Fields local_a8 [2];
  
  if ((DAT_028e6c26 & 1) == 0) {
    FUN_0083c778(&System.Action_TypeInfo);
    FUN_0083c778(&BanditCampsControl_TypeInfo);
    FUN_0083c778(&BasketContents_TypeInfo);
    FUN_0083c778(&byte[]_TypeInfo);
    FUN_0083c778(&char_TypeInfo);
    FUN_0083c778(&ChunkControl_TypeInfo);
    FUN_0083c778(&ChunkData_TypeInfo);
    FUN_0083c778(&ChunkElement_TypeInfo);
    FUN_0083c778(&CompanionController_TypeInfo);
    FUN_0083c778(&ConstructionControl_TypeInfo);
    FUN_0083c778(&CreatureStruct_TypeInfo);
    FUN_0083c778(&CustomTeleporterControl_TypeInfo);
    FUN_0083c778(&System.DateTime_TypeInfo);
    FUN_0083c778(&UnityEngine.Debug_TypeInfo);
    FUN_0083c778(&Method$System.Collections.Generic.Dictionary<string,-List<int>>.Add());
    FUN_0083c778(&Method$System.Collections.Generic.Dictionary<string,-Sprite>.Add());
    FUN_0083c778(&Method$System.Collections.Generic.Dictionary<string,-LandClaimChunkTimer>.Add());
    FUN_0083c778(&Method$System.Collections.Generic.Dictionary<string,-BanditCampInstance>.Add());
    FUN_0083c778(&Method$System.Collections.Generic.Dictionary<string,-ChunkData>.Add());
    FUN_0083c778(&Method$System.Collections.Generic.Dictionary<string,-List<int>>.Clear());
    FUN_0083c778(&Method$System.Collections.Generic.Dictionary<string,-ChunkData>.ContainsKey());
    FUN_0083c778(&Method$System.Collections.Generic.Dictionary<string,-List<int>>.ContainsKey());
    FUN_0083c778(&Method$System.Collections.Generic.Dictionary<string,-CreatureStruct>.ContainsKey()
                );
    FUN_0083c778(&Method$System.Collections.Generic.Dictionary<string,-Sprite>.ContainsKey());
    FUN_0083c778(&
                 Method$System.Collections.Generic.Dictionary<string,-LandClaimChunkTimer>.ContainsK ey()
                );
    FUN_0083c778(&
                 Method$System.Collections.Generic.Dictionary<string,-BanditCampInstance>.ContainsKe y()
                );
    FUN_0083c778(&Method$System.Collections.Generic.Dictionary<string,-OnlinePlayer>.ContainsKey());
    FUN_0083c778(&Method$System.Collections.Generic.Dictionary<string,-GameObject>.ContainsKey());
    FUN_0083c778(&Method$System.Collections.Generic.Dictionary<string,-ChunkData>.GetEnumerator());
    FUN_0083c778(&Method$System.Collections.Generic.Dictionary<string,-LandClaimChunkTimer>.Remove()
                );
    FUN_0083c778(&Method$System.Collections.Generic.Dictionary<string,-ChunkData>.Remove());
    FUN_0083c778(&Method$System.Collections.Generic.Dictionary<string,-List<int>>.Remove());
    FUN_0083c778(&Method$System.Collections.Generic.Dictionary<string,-CreatureStruct>.Remove());
    FUN_0083c778(&Method$System.Collections.Generic.Dictionary<string,-ChunkData>.get_Count());
    FUN_0083c778(&
                 Method$System.Collections.Generic.Dictionary<string,-BanditCampInstance>.get_Item()
                );
    FUN_0083c778(&Method$System.Collections.Generic.Dictionary<string,-GameObject>.get_Item());
    FUN_0083c778(&
                 Method$System.Collections.Generic.Dictionary<string,-LandClaimChunkTimer>.get_Item( )
                );
    FUN_0083c778(&Method$System.Collections.Generic.Dictionary<string,-OnlinePlayer>.get_Item());
    FUN_0083c778(&Method$System.Collections.Generic.Dictionary<string,-Sprite>.get_Item());
    FUN_0083c778(&Method$System.Collections.Generic.Dictionary<string,-ChunkData>.get_Item());
    FUN_0083c778(&Method$System.Collections.Generic.Dictionary<string,-List<int>>.get_Item());
    FUN_0083c778(&Method$System.Collections.Generic.Dictionary<string,-CreatureStruct>.get_Item());
    FUN_0083c778(&Method$System.Collections.Generic.Dictionary<string,-ChunkData>.set_Item());
    FUN_0083c778(&
                 Method$System.Collections.Generic.Dictionary<string,-LandClaimChunkTimer>.set_Item( )
                );
    FUN_0083c778(&Method$System.Collections.Generic.Dictionary<string,-Sprite>.set_Item());
    FUN_0083c778(&Method$System.Collections.Generic.List.Enumerator<string>.Dispose());
    FUN_0083c778(&Method$System.Collections.Generic.List.Enumerator<Friend>.Dispose());
    FUN_0083c778(&
                 Method$System.Collections.Generic.Dictionary.Enumerator<string,-ChunkData>.Dispose( )
                );
    FUN_0083c778(&Method$System.Collections.Generic.List.Enumerator<ActiveCompanion>.Dispose());
    FUN_0083c778(&
                 Method$System.Collections.Generic.Dictionary.Enumerator<string,-ChunkData>.MoveNext ()
                );
    FUN_0083c778(&Method$System.Collections.Generic.List.Enumerator<ActiveCompanion>.MoveNext());
    FUN_0083c778(&Method$System.Collections.Generic.List.Enumerator<Friend>.MoveNext());
    FUN_0083c778(&Method$System.Collections.Generic.List.Enumerator<string>.MoveNext());
    FUN_0083c778(&Method$System.Collections.Generic.List.Enumerator<string>.get_Current());
    FUN_0083c778(&Method$System.Collections.Generic.List.Enumerator<Friend>.get_Current());
    FUN_0083c778(&
                 Method$System.Collections.Generic.Dictionary.Enumerator<string,-ChunkData>.get_Curr ent()
                );
    FUN_0083c778(&Method$System.Collections.Generic.List.Enumerator<ActiveCompanion>.get_Current());
    FUN_0083c778(&FriendServerInterface_TypeInfo);
    FUN_0083c778(&FriendServerReceiver_TypeInfo);
    FUN_0083c778(&FriendServerSender_TypeInfo);
    FUN_0083c778(&GameController_TypeInfo);
    FUN_0083c778(&Method$UnityEngine.GameObject.GetComponent<Combatant>());
    FUN_0083c778(&Method$UnityEngine.GameObject.GetComponent<PerkReceiver>());
    FUN_0083c778(&Method$UnityEngine.GameObject.GetComponent<SharedCreature>());
    FUN_0083c778(&GameServerConnector_TypeInfo);
    FUN_0083c778(&GameServerInterface_TypeInfo);
    FUN_0083c778(&GameServerSender_TypeInfo);
    FUN_0083c778(&GameplayGUIControl_TypeInfo);
    FUN_0083c778(&int[]_TypeInfo);
    FUN_0083c778(&Method$System.Collections.Generic.KeyValuePair<string,-ChunkData>.get_Key());
    FUN_0083c778(&Method$System.Collections.Generic.KeyValuePair<string,-ChunkData>.get_Value());
    FUN_0083c778(&LandClaimControl_TypeInfo);
    FUN_0083c778(&Method$System.Collections.Generic.List<int>.Add());
    FUN_0083c778(&Method$System.Collections.Generic.List<string>.Add());
    FUN_0083c778(&Method$System.Collections.Generic.List<string>.Clear());
    FUN_0083c778(&Method$System.Collections.Generic.List<string>.Contains());
    FUN_0083c778(&Method$System.Collections.Generic.List<Friend>.GetEnumerator());
    FUN_0083c778(&Method$System.Collections.Generic.List<ActiveCompanion>.GetEnumerator());
    FUN_0083c778(&Method$System.Collections.Generic.List<string>.GetEnumerator());
    FUN_0083c778(&Method$System.Collections.Generic.List<int>.Remove());
    FUN_0083c778(&Method$System.Collections.Generic.List<GameObject>.Remove());
    FUN_0083c778(&Method$System.Collections.Generic.List<string>.Remove());
    FUN_0083c778(&Method$System.Collections.Generic.List<int>..ctor());
    FUN_0083c778(&Method$System.Collections.Generic.List<string>..ctor());
    FUN_0083c778(&Method$System.Collections.Generic.List<string>.get_Count());
    FUN_0083c778(&Method$System.Collections.Generic.List<CraftingSlot>.get_Item());
    FUN_0083c778(&Method$System.Collections.Generic.List<string>.get_Item());
    FUN_0083c778(&System.Collections.Generic.List<int>_TypeInfo);
    FUN_0083c778(&System.Collections.Generic.List<string>_TypeInfo);
    FUN_0083c778(&LootControl_TypeInfo);
    FUN_0083c778(&MinigameMenu_TypeInfo);
    FUN_0083c778(&MobControl_TypeInfo);
    FUN_0083c778(&MusicBoxControl_TypeInfo);
    FUN_0083c778(&UnityEngine.Object_TypeInfo);
    FUN_0083c778(&OnlinePlayerData_TypeInfo);
    FUN_0083c778(&OnlineTeleporter_TypeInfo);
    FUN_0083c778(&Packet_TypeInfo);
    FUN_0083c778(&System.IO.Path_TypeInfo);
    FUN_0083c778(&PerkControl_TypeInfo);
    FUN_0083c778(&PerkData_TypeInfo);
    FUN_0083c778(&PlayerData_TypeInfo);
    FUN_0083c778(&PoolGameControl_TypeInfo);
    FUN_0083c778(&PoolGameRecording_TypeInfo);
    FUN_0083c778(&PopupControl_TypeInfo);
    FUN_0083c778(&QuestControl_TypeInfo);
    FUN_0083c778(&UnityEngine.SceneManagement.SceneManager_TypeInfo);
    FUN_0083c778(&Startup_TypeInfo);
    FUN_0083c778(&string[]_TypeInfo);
    FUN_0083c778(&UnityEngine.Texture2D_TypeInfo);
    FUN_0083c778(&System.TimeSpan_TypeInfo);
    FUN_0083c778(&TradingTableControl_TypeInfo);
    FUN_0083c778(&Method$GameServerReceiver.<>c__DisplayClass18_0.<OnReceive>b__0());
    FUN_0083c778(&Method$GameServerReceiver.<>c__DisplayClass18_0.<OnReceive>b__1());
    FUN_0083c778(&GameServerReceiver.<>c__DisplayClass18_0_TypeInfo);
    FUN_0083c778(&WindowControl_TypeInfo);
    FUN_0083c778(&ZoneDataControl_TypeInfo);
    FUN_0083c778(&StringLiteral_4412);
    FUN_0083c778(&StringLiteral_1418);
    FUN_0083c778(&StringLiteral_9947);
    FUN_0083c778(&StringLiteral_622);
    FUN_0083c778(&StringLiteral_9503);
    FUN_0083c778(&StringLiteral_1421);
    FUN_0083c778(&StringLiteral_4499);
    FUN_0083c778(&StringLiteral_1409);
    FUN_0083c778(&StringLiteral_5977);
    FUN_0083c778(&StringLiteral_1082);
    FUN_0083c778(&StringLiteral_6442);
    FUN_0083c778(&StringLiteral_9917);
    FUN_0083c778(&StringLiteral_3346);
    FUN_0083c778(&StringLiteral_10841);
    FUN_0083c778(&StringLiteral_1424);
    FUN_0083c778(&StringLiteral_8671);
    FUN_0083c778(&StringLiteral_3327);
    FUN_0083c778(&StringLiteral_1357);
    FUN_0083c778(&StringLiteral_736);
    FUN_0083c778(&StringLiteral_13258);
    FUN_0083c778(&StringLiteral_1583);
    FUN_0083c778(&StringLiteral_13671);
    FUN_0083c778(&StringLiteral_1437);
    FUN_0083c778(&StringLiteral_9354);
    FUN_0083c778(&StringLiteral_6647);
    FUN_0083c778(&StringLiteral_14261);
    FUN_0083c778(&StringLiteral_1608);
    FUN_0083c778(&StringLiteral_1776);
    FUN_0083c778(&StringLiteral_1565);
    FUN_0083c778(&StringLiteral_9643);
    FUN_0083c778(&StringLiteral_820);
    FUN_0083c778(&StringLiteral_1934);
    FUN_0083c778(&StringLiteral_1311);
    FUN_0083c778(&StringLiteral_1398);
    FUN_0083c778(&StringLiteral_14006);
    FUN_0083c778(&StringLiteral_1416);
    FUN_0083c778(&StringLiteral_9499);
    FUN_0083c778(&StringLiteral_5465);
    FUN_0083c778(&StringLiteral_1);
    FUN_0083c778(&StringLiteral_6100);
    FUN_0083c778(&StringLiteral_6902);
    FUN_0083c778(&StringLiteral_1417);
    FUN_0083c778(&StringLiteral_1338);
    FUN_0083c778(&StringLiteral_13664);
    FUN_0083c778(&chat_log_TypeInfo);
    FUN_0083c778(&inventory_ctr_TypeInfo);
    DAT_028e6c26 = 1;
  }
  local_a8[0].m_Handle = 0;
  local_b8 = 0;
  uStack_b0 = 0;
  local_bc = 0;
  local_d0 = 0;
  SStack_c8._dateData = 0;
  local_e0.rgctx_data = (Il2CppRGCTXData *)0x0;
  uStack_d8 = 0;
  local_f0 = (undefined1  [8])0x0;
  ppIStack_e8 = (Il2CppType **)0x0;
  local_108 = (Il2CppType **)0x0;
  _Stack_100.rgctx_data = (Il2CppRGCTXData *)0x0;
  local_110 = (undefined1  [8])0x0;
  local_120 = (Il2CppClass *)0x0;
  local_150.rgctx_data = (Il2CppRGCTXData *)0x0;
  _Stack_148.genericMethod = (void *)0x0;
  ppIStack_138 = (Il2CppType **)0x0;
  local_140 = (System_Collections_Generic_List_T__o *)0x0;
  pcStack_128 = (char *)0x0;
  _Stack_130.rgctx_data = (Il2CppRGCTXData *)0x0;
  auStack_188._40_8_ = (System_Collections_Generic_List_T__o *)0x0;
  ppIStack_158 = (Il2CppType **)0x0;
  if ((UnityEngine.SceneManagement.SceneManager_TypeInfo->_2).cctor_finished == 0) {
    thunk_FUN_008bc8d8();
  }
  local_a8[0].m_Handle =
       (int32_t)UnityEngine.SceneManagement.SceneManager$$GetActiveScene((MethodInfo *)0x0);
  pSVar24 = UnityEngine.SceneManagement.Scene$$get_name
                      ((UnityEngine_SceneManagement_Scene_Fields)(int32_t)local_a8,(MethodInfo *)0x0
                      );
  bVar5 = System.String$$op_Inequality(pSVar24,StringLiteral_4412,(MethodInfo *)0x0);
  if (bVar5) {
    pGVar90 = GameServerConnector_TypeInfo->static_fields->Instance;
    if (pGVar90 != (GameServerConnector_o *)0x0) {
      pCVar25 = (pGVar90->fields).game_server_connection;
      if (pCVar25 == (Connection_o *)0x0) {
        return;
      }
      Connection$$Disconnect(pCVar25,(MethodInfo *)0x0);
      return;
    }
    goto LAB_00958368;
  }
  if (incoming == (Packet_o *)0x0) goto LAB_00958368;
  uVar6 = Packet$$GetByte(incoming,(MethodInfo *)0x0);
  method_06 = &Method$System.Collections.Generic.List<string>.get_Item();
  method_08 = &Method$System.Collections.Generic.List<string>.Remove();
  method_07 = &inventory_ctr_TypeInfo;
  method_04 = &Startup_TypeInfo;
  switch(uVar6) {
  case 1:
    if (DAT_028e6c86 == '\0') {
      FUN_0083c778(&GameServerConnector_TypeInfo);
      DAT_028e6c86 = '\x01';
    }
    pGVar90 = GameServerConnector_TypeInfo->static_fields->Instance;
    if ((System.DateTime_TypeInfo->_2).cctor_finished == 0) {
      thunk_FUN_008bc8d8();
    }
    SVar84 = System.DateTime$$get_UtcNow((MethodInfo *)0x0);
    if (pGVar90 == (GameServerConnector_o *)0x0) goto LAB_00958368;
    (pGVar90->fields).last_server_ping.fields._dateData = (uint64_t)SVar84.fields._dateData;
    break;
  case 2:
    pFVar83 = FriendServerSender_TypeInfo->static_fields->Instance;
    if (pFVar83 != (FriendServerSender_o *)0x0) {
      (pFVar83->fields).sending_request_of_some_sort = false;
      if (DAT_028e6c86 == '\0') {
        FUN_0083c778(&GameServerConnector_TypeInfo);
        DAT_028e6c86 = '\x01';
      }
      pGVar90 = GameServerConnector_TypeInfo->static_fields->Instance;
      pSVar24 = Packet$$GetString(incoming,(MethodInfo *)0x0);
      if (pGVar90 != (GameServerConnector_o *)0x0) {
        ppSVar121 = &(pGVar90->fields).server_name;
        *ppSVar121 = pSVar24;
        thunk_FUN_008c6b48(ppSVar121,pSVar24);
        if (DAT_028e6c86 == '\0') {
          FUN_0083c778(&GameServerConnector_TypeInfo);
          DAT_028e6c86 = '\x01';
        }
        pGVar90 = GameServerConnector_TypeInfo->static_fields->Instance;
        uVar6 = Packet$$GetByte(incoming,(MethodInfo *)0x0);
        if (pGVar90 != (GameServerConnector_o *)0x0) {
          (pGVar90->fields).is_host_cached = uVar6 == 1;
          Packet$$GetByte(incoming,(MethodInfo *)0x0);
          if (DAT_028e6c86 == '\0') {
            FUN_0083c778(&GameServerConnector_TypeInfo);
            DAT_028e6c86 = '\x01';
          }
          pGVar90 = GameServerConnector_TypeInfo->static_fields->Instance;
          if (pGVar90 != (GameServerConnector_o *)0x0) {
            (pGVar90->fields).pvp_enabled = false;
            if (DAT_028e6c85 == '\0') {
              FUN_0083c778(&GameServerSender_TypeInfo);
              DAT_028e6c85 = '\x01';
            }
            pGVar106 = GameServerSender_TypeInfo->static_fields->Instance;
            pSVar24 = Packet$$GetString(incoming,(MethodInfo *)0x0);
            if (pGVar106 != (GameServerSender_o *)0x0) {
              ppSVar121 = &(pGVar106->fields).packet_validator_code;
              *ppSVar121 = pSVar24;
              thunk_FUN_008c6b48(ppSVar121,pSVar24);
              if (DAT_028e6c85 == '\0') {
                FUN_0083c778(&GameServerSender_TypeInfo);
                DAT_028e6c85 = '\x01';
              }
              pGVar106 = GameServerSender_TypeInfo->static_fields->Instance;
              iVar10 = Packet$$GetShort(incoming,(MethodInfo *)0x0);
              if (pGVar106 != (GameServerSender_o *)0x0) {
                (pGVar106->fields).packet_validator_total_variation = (int)iVar10;
                pSVar58 = (System_Collections_Generic_List_object__o *)
                          thunk_FUN_008184f0(System.Collections.Generic.List<string>_TypeInfo);
                if (pSVar58 != (System_Collections_Generic_List_object__o *)0x0) {
                  System.Collections.Generic.List<object>$$.ctor
                            (pSVar58,Method$System.Collections.Generic.List<string>..ctor());
                  if (DAT_028e6c86 == '\0') {
                    FUN_0083c778(&GameServerConnector_TypeInfo);
                    DAT_028e6c86 = '\x01';
                  }
                  pGVar90 = GameServerConnector_TypeInfo->static_fields->Instance;
                  iVar10 = Packet$$GetShort(incoming,(MethodInfo *)0x0);
                  if (pGVar90 != (GameServerConnector_o *)0x0) {
                    (pGVar90->fields).n_others_in_game = (int)iVar10;
                    pMVar102 = extraout_x1_07;
                    if (DAT_028e6c86 == '\0') {
                      FUN_0083c778(&GameServerConnector_TypeInfo);
                      DAT_028e6c86 = '\x01';
                      pMVar102 = extraout_x1_08;
                    }
                    pGVar90 = GameServerConnector_TypeInfo->static_fields->Instance;
                    if (pGVar90 != (GameServerConnector_o *)0x0) {
                      if (((pGVar90->fields).is_host_cached == false) ||
                         ((pGVar90->fields).n_others_in_game == 0)) {
code_r0x00955878:
                        pFVar83 = FriendServerSender_TypeInfo->static_fields->Instance;
                        if (pFVar83 != (FriendServerSender_o *)0x0) {
                          FriendServerSender$$UpdateWorldString(pFVar83,pMVar102);
                          pMVar102 = extraout_x1_12;
                          if (DAT_028e6c86 == '\0') {
                            FUN_0083c778(&GameServerConnector_TypeInfo);
                            DAT_028e6c86 = '\x01';
                            pMVar102 = extraout_x1_13;
                          }
                          pGVar90 = GameServerConnector_TypeInfo->static_fields->Instance;
                          if (pGVar90 != (GameServerConnector_o *)0x0) {
                            (pGVar90->fields).completely_logged_in = true;
                            GameServerConnector$$StartPinging(pGVar90,pMVar102);
                            pPVar52 = PopupControl_TypeInfo->static_fields->Instance;
                            if (pPVar52 != (PopupControl_o *)0x0) {
                              PopupControl$$HideAll(pPVar52,(MethodInfo *)0x0);
                              pMVar102 = extraout_x1_14;
                              if (DAT_028e6c86 == '\0') {
                                FUN_0083c778(&GameServerConnector_TypeInfo);
                                DAT_028e6c86 = '\x01';
                                pMVar102 = extraout_x1_15;
                              }
                              pGVar90 = GameServerConnector_TypeInfo->static_fields->Instance;
                              if (pGVar90 != (GameServerConnector_o *)0x0) {
                                if ((pGVar90->fields).is_host_cached == false) {
                                  pSVar24 = (pGVar90->fields).server_name;
                                  if (pSVar24 == (System_String_o *)0x0) goto LAB_00958368;
                                  pSVar24 = System.String$$Replace
                                                      (pSVar24,StringLiteral_736,
                                                       (System_String_o *)StringLiteral_1.rgctx_data
                                                       ,(MethodInfo *)0x0);
                                  pSVar24 = System.String$$Concat
                                                      (_StringLiteral_9354,pSVar24,
                                                       _StringLiteral_622,(MethodInfo *)0x0);
                                  pFVar108 = FriendServerInterface_TypeInfo->static_fields->Instance
                                  ;
                                  if (pFVar108 == (FriendServerInterface_o *)0x0) goto LAB_00958368;
                                  method_06 = (MethodInfo_F19A34 **)
                                              (pFVar108->fields).default_server_icon;
                                  pcVar80 = (chat_log_o *)thunk_FUN_008184f0(chat_log_TypeInfo);
                                  if (pcVar80 == (chat_log_o *)0x0) goto LAB_00958368;
                                  chat_log$$.ctor(pcVar80,pSVar24,
                                                  (System_String_o *)StringLiteral_1.rgctx_data,
                                                  (UnityEngine_Sprite_o *)method_06,true,
                                                  (
                                                  System_Collections_Generic_Dictionary_string__stri ng__o
                                                  *)0x0,(MethodInfo *)0x0);
                                  if (DAT_028e6c84 == '\0') {
                                    FUN_0083c778(&GameServerInterface_TypeInfo);
                                    DAT_028e6c84 = '\x01';
                                  }
                                  pGVar95 = GameServerInterface_TypeInfo->static_fields->Instance;
                                  if ((pGVar95 == (GameServerInterface_o *)0x0) ||
                                     (pCVar81 = (pGVar95->fields).game_chat,
                                     pCVar81 == (ChatCollection_o *)0x0)) goto LAB_00958368;
                                  pMVar102 = (MethodInfo *)0x0;
                                  ChatCollection$$AddLog(pCVar81,pcVar80,(MethodInfo *)0x0);
                                  if (DAT_028e6c84 == '\0') {
                                    FUN_0083c778(&GameServerInterface_TypeInfo);
                                    DAT_028e6c84 = '\x01';
                                  }
                                  pGVar95 = GameServerInterface_TypeInfo->static_fields->Instance;
                                  if (pGVar95 == (GameServerInterface_o *)0x0) goto LAB_00958368;
                                  GameServerInterface$$GameChatReceived(pGVar95,pcVar80,pMVar102);
                                  pMVar102 = extraout_x1_16;
                                  if (DAT_028e6c86 == '\0') {
                                    FUN_0083c778(&GameServerConnector_TypeInfo);
                                    DAT_028e6c86 = '\x01';
                                    pMVar102 = extraout_x1_17;
                                  }
                                }
                                pGVar90 = GameServerConnector_TypeInfo->static_fields->Instance;
                                if (pGVar90 != (GameServerConnector_o *)0x0) {
                                  if ((pGVar90->fields).is_host_cached != false) {
                                    System.Collections.Generic.List<object>$$GetEnumerator
                                              ((System_Collections_Generic_List_Enumerator_T__o *)
                                               auStack_188,pSVar58,
                                               Method$System.Collections.Generic.List<string>.GetEnu merator()
                                              );
                                    ppIStack_e8 = (Il2CppType **)auStack_188._8_8_;
                                    local_f0 = (undefined1  [8])auStack_188._0_8_;
                                    local_e0 = (_union_13)auStack_188._16_8_;
                                    while (bVar5 = 
                                                  System.Collections.Generic.List.Enumerator<object> $$MoveNext
                                                            ((
                                                  System_Collections_Generic_List_Enumerator_T__o *)
                                                  local_f0,
                                                  Method$System.Collections.Generic.List.Enumerator< string>.MoveNext()
                                                  ), _Var53 = local_e0, bVar5) {
                                      if (DAT_028e6c84 == '\0') {
                                        FUN_0083c778(&GameServerInterface_TypeInfo);
                                        DAT_028e6c84 = '\x01';
                                      }
                                      pGVar95 = GameServerInterface_TypeInfo->static_fields->
                                                Instance;
                                      if (pGVar95 == (GameServerInterface_o *)0x0) {
                    /* WARNING: Subroutine does not return */
                                        FUN_0083c89c();
                                      }
                                      GameServerInterface$$ShowPlayerLogInOrOut
                                                (pGVar95,(System_String_o *)_Var53.rgctx_data,1,
                                                 (MethodInfo *)method_06);
                                    }
                                    System.Collections.Generic.List.Enumerator<object>$$Dispose
                                              ((System_Collections_Generic_List_Enumerator_T__o *)
                                               local_f0,
                                               Method$System.Collections.Generic.List.Enumerator<str ing>.Dispose()
                                              );
                                    pMVar102 = extraout_x1_18;
                                  }
                                  if (DAT_028e6c86 == '\0') {
                                    FUN_0083c778(&GameServerConnector_TypeInfo);
                                    DAT_028e6c86 = '\x01';
                                    pMVar102 = extraout_x1_19;
                                  }
                                  pGVar90 = GameServerConnector_TypeInfo->static_fields->Instance;
                                  if (pGVar90 != (GameServerConnector_o *)0x0) {
                                    if ((pGVar90->fields).is_host_cached == false) {
                                      GameServerReceiver$$ClearPreviousMap(__this,pMVar102);
                                      pMVar102 = extraout_x1_23;
                                    }
                                    else {
                                      pSVar107 = (__this->fields).unique_ids_given_away;
                                      if (pSVar107 ==
                                          (System_Collections_Generic_Dictionary_string__List_int___ o
                                           *)0x0) goto LAB_00958368;
                                      System.Collections.Generic.Dictionary<>$$Clear
                                                (pSVar107,
                                                 Method$System.Collections.Generic.Dictionary<string ,-List<int>>.Clear()
                                                );
                                      pMVar102 = extraout_x1_20;
                                    }
                                    if (DAT_028e6c85 == '\0') {
                                      FUN_0083c778(&GameServerSender_TypeInfo);
                                      DAT_028e6c85 = '\x01';
                                      pMVar102 = extraout_x1_24;
                                    }
                                    pGVar106 = GameServerSender_TypeInfo->static_fields->Instance;
                                    if (pGVar106 != (GameServerSender_o *)0x0) {
                                      GameServerSender$$SendInitialPlayerData(pGVar106,pMVar102);
                                      return;
                                    }
                                  }
                                }
                              }
                            }
                          }
                        }
                      }
                      else {
                        iVar19 = 0;
                        cVar89 = '\x01';
                        while( true ) {
                          if (cVar89 == '\0') {
                            FUN_0083c778(&GameServerConnector_TypeInfo);
                            DAT_028e6c86 = '\x01';
                            pMVar102 = extraout_x1_09;
                          }
                          pGVar90 = GameServerConnector_TypeInfo->static_fields->Instance;
                          if (pGVar90 == (GameServerConnector_o *)0x0) break;
                          if ((pGVar90->fields).n_others_in_game <= iVar19) goto code_r0x00955878;
                          pSVar24 = Packet$$GetString(incoming,(MethodInfo *)0x0);
                          lVar61 = Method$System.Collections.Generic.List<string>.Add();
                          pSVar114 = (pSVar58->fields)._items;
                          (pSVar58->fields)._version = (pSVar58->fields)._version + 1;
                          if (pSVar114 == (System_Object_array *)0x0) break;
                          uVar18 = (pSVar58->fields)._size;
                          if (uVar18 < (uint)pSVar114->max_length) {
                            (pSVar58->fields)._size = uVar18 + 1;
                            pSVar114->m_Items[(int)uVar18] = (Il2CppObject *)pSVar24;
                            thunk_FUN_008c6b48();
                            pMVar102 = extraout_x1_10;
                          }
                          else {
                            (**(code **)(*(long *)(*(long *)(*(long *)(lVar61 + 0x20) + 0xc0) + 0x58
                                                  ) + 8))(pSVar58);
                            pMVar102 = extraout_x1_11;
                          }
                          iVar19 = iVar19 + 1;
                          cVar89 = DAT_028e6c86;
                        }
                      }
                    }
                  }
                }
              }
            }
          }
        }
      }
    }
    goto LAB_00958368;
  case 4:
    pSVar24 = Packet$$GetString(incoming,(MethodInfo *)0x0);
    pPVar34 = (Packet_o *)thunk_FUN_008184f0(Packet_TypeInfo);
    if (pPVar34 == (Packet_o *)0x0) goto LAB_00958368;
    Packet$$.ctor(pPVar34,(MethodInfo *)0x0);
    Packet$$PutByte(pPVar34,4,(MethodInfo *)0x0);
    Packet$$PutString(pPVar34,pSVar24,(MethodInfo *)0x0);
    iVar19 = 0x19;
    do {
      pCVar104 = ConstructionControl_TypeInfo->static_fields->Instance;
      if (pCVar104 == (ConstructionControl_o *)0x0) goto LAB_00958368;
      iVar17 = ConstructionControl$$GetNewUniqueId(pCVar104,true,(MethodInfo *)0x0);
      Packet$$PutLong(pPVar34,iVar17,(MethodInfo *)0x0);
      pSVar107 = (__this->fields).unique_ids_given_away;
      if (pSVar107 == (System_Collections_Generic_Dictionary_string__List_int___o *)0x0)
      goto LAB_00958368;
      uVar32 = System.Collections.Generic.Dictionary<>$$ContainsKey
                         (pSVar107,pSVar24,
                          _Method$System.Collections.Generic.Dictionary<string,-List<int>>.ContainsK ey()
                         );
      if ((uVar32 & 1) == 0) {
        pSVar107 = (__this->fields).unique_ids_given_away;
        pSVar72 = (System_Collections_Generic_List_int__o *)
                  thunk_FUN_008184f0(System.Collections.Generic.List<int>_TypeInfo);
        if ((pSVar72 == (System_Collections_Generic_List_int__o *)0x0) ||
           (System.Collections.Generic.List<int>$$.ctor
                      (pSVar72,Method$System.Collections.Generic.List<int>..ctor()),
           pSVar107 == (System_Collections_Generic_Dictionary_string__List_int___o *)0x0))
        goto LAB_00958368;
        System.Collections.Generic.Dictionary<>$$Add
                  (pSVar107,pSVar24,pSVar72,
                   _Method$System.Collections.Generic.Dictionary<string,-List<int>>.Add());
      }
      pSVar107 = (__this->fields).unique_ids_given_away;
      if (pSVar107 == (System_Collections_Generic_Dictionary_string__List_int___o *)0x0)
      goto LAB_00958368;
      auVar132 = System.Collections.Generic.Dictionary<>$$get_Item
                           (pSVar107,pSVar24,
                            _Method$System.Collections.Generic.Dictionary<string,-List<int>>.get_Ite m()
                           );
      lVar61 = Method$System.Collections.Generic.List<int>.Add();
      lVar42 = auVar132._0_8_;
      if (lVar42 == 0) goto LAB_00958368;
      lVar96 = *(long *)(lVar42 + 0x10);
      *(int *)(lVar42 + 0x1c) = *(int *)(lVar42 + 0x1c) + 1;
      if (lVar96 == 0) goto LAB_00958368;
      uVar18 = *(uint *)(lVar42 + 0x18);
      if (uVar18 < *(uint *)(lVar96 + 0x18)) {
        *(uint *)(lVar42 + 0x18) = uVar18 + 1;
        *(int32_t *)(lVar96 + (long)(int)uVar18 * 4 + 0x20) = iVar17;
      }
      else {
        auVar132 = (**(code **)(*(long *)(*(long *)(*(long *)(lVar61 + 0x20) + 0xc0) + 0x58) + 8))
                             (lVar42,iVar17);
      }
      iVar19 = iVar19 + -1;
    } while (iVar19 != 0);
    pCVar25 = GameServerReceiver$$get_connection(auVar132._0_8_,auVar132._8_8_);
    if (pCVar25 == (Connection_o *)0x0) goto LAB_00958368;
    iVar17 = 0;
    goto code_r0x00957938;
  case 5:
    iVar10 = Packet$$GetShort(incoming,(MethodInfo *)0x0);
    iVar19 = (int)iVar10;
    if (0 < iVar19) {
      do {
        pCVar104 = ConstructionControl_TypeInfo->static_fields->Instance;
        if (pCVar104 == (ConstructionControl_o *)0x0) goto LAB_00958368;
        pSVar72 = (pCVar104->fields).online_unique_ids_;
        iVar17 = Packet$$GetLong(incoming,(MethodInfo *)0x0);
        lVar61 = Method$System.Collections.Generic.List<int>.Add();
        if (pSVar72 == (System_Collections_Generic_List_int__o *)0x0) goto LAB_00958368;
        pSVar75 = (pSVar72->fields)._items;
        (pSVar72->fields)._version = (pSVar72->fields)._version + 1;
        if (pSVar75 == (System_Int32_array *)0x0) goto LAB_00958368;
        uVar18 = (pSVar72->fields)._size;
        if (uVar18 < (uint)pSVar75->max_length) {
          (pSVar72->fields)._size = uVar18 + 1;
          pSVar75->m_Items[(int)uVar18] = iVar17;
        }
        else {
          method_04 = *(Startup_c ***)(*(long *)(*(long *)(lVar61 + 0x20) + 0xc0) + 0x58);
          (*((MethodInfo *)method_04)->virtualMethodPointer)(pSVar72);
        }
        iVar19 = iVar19 + -1;
      } while (iVar19 != 0);
    }
    GameServerReceiver$$ReceiveDaynight(__this,incoming,(MethodInfo *)method_04);
    pSVar27 = (__this->fields).disabled_perks;
    if (pSVar27 != (System_Collections_Generic_List_string__o *)0x0) {
      uVar18 = (pSVar27->fields)._size;
      pMVar102 = (MethodInfo *)(ulong)uVar18;
      (pSVar27->fields)._size = 0;
      (pSVar27->fields)._version = (pSVar27->fields)._version + 1;
      if (0 < (int)uVar18) {
        method_06 = (MethodInfo_F19A34 **)0x0;
        System.Array$$Clear((System_Array_o *)(pSVar27->fields)._items,0,uVar18,(MethodInfo *)0x0);
      }
      iVar10 = Packet$$GetShort(incoming,(MethodInfo *)0x0);
      iVar19 = (int)iVar10;
      if (0 < iVar19) {
        do {
          pSVar27 = (__this->fields).disabled_perks;
          pSVar24 = Packet$$GetString(incoming,(MethodInfo *)0x0);
          lVar61 = Method$System.Collections.Generic.List<string>.Add();
          if (pSVar27 == (System_Collections_Generic_List_string__o *)0x0) goto LAB_00958368;
          pSVar115 = (pSVar27->fields)._items;
          (pSVar27->fields)._version = (pSVar27->fields)._version + 1;
          if (pSVar115 == (System_String_array *)0x0) goto LAB_00958368;
          uVar18 = (pSVar27->fields)._size;
          if (uVar18 < (uint)pSVar115->max_length) {
            (pSVar27->fields)._size = uVar18 + 1;
            pSVar115->m_Items[(int)uVar18] = pSVar24;
            thunk_FUN_008c6b48();
          }
          else {
            pMVar102 = *(MethodInfo **)(*(long *)(*(long *)(lVar61 + 0x20) + 0xc0) + 0x58);
            (*pMVar102->virtualMethodPointer)(pSVar27);
          }
          iVar19 = iVar19 + -1;
        } while (iVar19 != 0);
      }
      if (DAT_028e6c86 == '\0') {
        FUN_0083c778(&GameServerConnector_TypeInfo);
        DAT_028e6c86 = '\x01';
      }
      pGVar90 = GameServerConnector_TypeInfo->static_fields->Instance;
      uVar6 = Packet$$GetByte(incoming,(MethodInfo *)0x0);
      if (pGVar90 != (GameServerConnector_o *)0x0) {
        (pGVar90->fields).is_moderator = uVar6 == 1;
        if ((CompanionController_TypeInfo->_2).cctor_finished == 0) {
          thunk_FUN_008bc8d8();
        }
        pCVar49 = CompanionController_TypeInfo->static_fields->Instance;
        uVar6 = Packet$$GetByte(incoming,(MethodInfo *)0x0);
        if (pCVar49 != (CompanionController_o *)0x0) {
          (pCVar49->fields).max_personal_companions_right_now = (uint)uVar6;
          pCVar49 = CompanionController_TypeInfo->static_fields->Instance;
          if (pCVar49 != (CompanionController_o *)0x0) {
            CompanionController$$RecreateAllCompanions(pCVar49,(MethodInfo *)0x0);
            pCVar49 = CompanionController_TypeInfo->static_fields->Instance;
            if ((pCVar49 != (CompanionController_o *)0x0) &&
               (__this_10 = (pCVar49->fields).active_companions,
               __this_10 != (System_Collections_Generic_List_ActiveCompanion__o *)0x0)) {
              System.Collections.Generic.List<object>$$GetEnumerator
                        ((System_Collections_Generic_List_Enumerator_T__o *)auStack_188,
                         (System_Collections_Generic_List_object__o *)__this_10,
                         Method$System.Collections.Generic.List<ActiveCompanion>.GetEnumerator());
              local_108 = (Il2CppType **)auStack_188._8_8_;
              local_110 = (undefined1  [8])auStack_188._0_8_;
              _Stack_100 = (_union_13)auStack_188._16_8_;
              while (bVar5 = System.Collections.Generic.List.Enumerator<object>$$MoveNext
                                       ((System_Collections_Generic_List_Enumerator_T__o *)local_110
                                        ,
                                        Method$System.Collections.Generic.List.Enumerator<ActiveComp anion>.MoveNext()
                                       ), _Var53 = _Stack_100, bVar5) {
                if (DAT_028e6c85 == '\0') {
                  FUN_0083c778(&GameServerSender_TypeInfo);
                  DAT_028e6c85 = '\x01';
                }
                if (_Var53.rgctx_data == (Il2CppRGCTXData *)0x0) {
                    /* WARNING: Subroutine does not return */
                  FUN_0083c89c();
                }
                pGVar106 = GameServerSender_TypeInfo->static_fields->Instance;
                pSVar24 = ActiveCompanion$$get_combat_name
                                    ((ActiveCompanion_o *)_Var53.rgctx_data,(MethodInfo *)0x0);
                if (pGVar106 == (GameServerSender_o *)0x0) {
                    /* WARNING: Subroutine does not return */
                  FUN_0083c89c();
                }
                GameServerSender$$SendCreatedLocalMob(pGVar106,pSVar24,pMVar102);
              }
              System.Collections.Generic.List.Enumerator<object>$$Dispose
                        ((System_Collections_Generic_List_Enumerator_T__o *)local_110,
                         Method$System.Collections.Generic.List.Enumerator<ActiveCompanion>.Dispose( )
                        );
              pGVar76 = GameplayGUIControl_TypeInfo->static_fields->Instance;
              if (pGVar76 != (GameplayGUIControl_o *)0x0) {
                GameplayGUIControl$$HideGameplayGui(pGVar76,(MethodInfo *)0x0);
                pGVar76 = GameplayGUIControl_TypeInfo->static_fields->Instance;
                if (pGVar76 != (GameplayGUIControl_o *)0x0) {
                  (pGVar76->fields).curr_GUI = 0;
                  GameplayGUIControl$$ShowGameplayGui(pGVar76,(MethodInfo *)0x0);
                  uVar6 = Packet$$GetByte(incoming,(MethodInfo *)0x0);
                  if (uVar6 == 0) {
                    if ((GameController_TypeInfo->_2).cctor_finished == 0) {
                      thunk_FUN_008bc8d8();
                    }
                    pGVar93 = GameController_TypeInfo->static_fields->Instance;
                    if (DAT_028e6c86 == '\0') {
                      FUN_0083c778(&GameServerConnector_TypeInfo);
                      DAT_028e6c86 = '\x01';
                    }
                    pGVar90 = GameServerConnector_TypeInfo->static_fields->Instance;
                    if ((pGVar90 == (GameServerConnector_o *)0x0) ||
                       (pGVar93 == (GameController_o *)0x0)) goto LAB_00958368;
                    UVar129 = GameController$$GetSavedPlayerPositionOnServer
                                        (pGVar93,(pGVar90->fields).server_name,(MethodInfo *)0x0);
                    pGVar93 = GameController_TypeInfo->static_fields->Instance;
                    if (DAT_028e6c86 == '\0') {
                      FUN_0083c778(&GameServerConnector_TypeInfo);
                      DAT_028e6c86 = '\x01';
                    }
                    pGVar90 = GameServerConnector_TypeInfo->static_fields->Instance;
                    if ((pGVar90 == (GameServerConnector_o *)0x0) ||
                       (pGVar93 == (GameController_o *)0x0)) goto LAB_00958368;
                    pSVar24 = GameController$$GetSavedPlayerZoneOnServer
                                        (pGVar93,(pGVar90->fields).server_name,(MethodInfo *)0x0);
                    if (DAT_028e6c85 == '\0') {
                      FUN_0083c778(&GameServerSender_TypeInfo);
                      DAT_028e6c85 = '\x01';
                    }
                    pGVar106 = GameServerSender_TypeInfo->static_fields->Instance;
                    if (pGVar106 == (GameServerSender_o *)0x0) goto LAB_00958368;
                    GameServerSender$$RequestZoneData
                              (pGVar106,pSVar24,3,UVar129,(MethodInfo *)method_06);
                  }
                  if (DAT_028e6c86 == '\0') {
                    FUN_0083c778(&GameServerConnector_TypeInfo);
                    DAT_028e6c86 = '\x01';
                  }
                  pGVar90 = GameServerConnector_TypeInfo->static_fields->Instance;
                  uVar6 = Packet$$GetByte(incoming,(MethodInfo *)0x0);
                  if (pGVar90 != (GameServerConnector_o *)0x0) {
                    (pGVar90->fields).pvp_enabled = uVar6 == 1;
                    Packet$$GetByte(incoming,(MethodInfo *)0x0);
                    return;
                  }
                }
              }
            }
          }
        }
      }
    }
    goto LAB_00958368;
  case 6:
    pSVar24 = Packet$$GetString(incoming,(MethodInfo *)0x0);
    pSVar29 = Packet$$GetString(incoming,(MethodInfo *)0x0);
    pSVar63 = Packet$$GetString(incoming,(MethodInfo *)0x0);
    uVar6 = Packet$$GetByte(incoming,(MethodInfo *)0x0);
    if (uVar6 == 1) {
      pPVar64 = PlayerData_TypeInfo->static_fields->Instance;
      if (pPVar64 == (PlayerData_o *)0x0) goto LAB_00958368;
      pSVar65 = PlayerData$$GetGlobalString(pPVar64,StringLiteral_14006,(MethodInfo *)0x0);
      bVar5 = System.String$$op_Equality(pSVar24,pSVar65,(MethodInfo *)0x0);
      if (!bVar5) {
        if ((FriendServerReceiver_TypeInfo->_2).cctor_finished == 0) {
          thunk_FUN_008bc8d8();
        }
        pFVar101 = FriendServerReceiver_TypeInfo->static_fields->Instance;
        if ((pFVar101 == (FriendServerReceiver_o *)0x0) ||
           (__this_11 = (pFVar101->fields).friends,
           __this_11 == (System_Collections_Generic_List_Friend__o *)0x0)) goto LAB_00958368;
        System.Collections.Generic.List<object>$$GetEnumerator
                  ((System_Collections_Generic_List_Enumerator_T__o *)auStack_188,
                   (System_Collections_Generic_List_object__o *)__this_11,
                   Method$System.Collections.Generic.List<Friend>.GetEnumerator());
        ppIStack_158 = (Il2CppType **)auStack_188._8_8_;
        auStack_188._40_8_ = auStack_188._0_8_;
        local_150 = (_union_13)auStack_188._16_8_;
        do {
          bVar5 = System.Collections.Generic.List.Enumerator<object>$$MoveNext
                            ((System_Collections_Generic_List_Enumerator_T__o *)(auStack_188 + 0x28)
                             ,Method$System.Collections.Generic.List.Enumerator<Friend>.MoveNext());
          if (!bVar5) {
            System.Collections.Generic.List.Enumerator<object>$$Dispose
                      ((System_Collections_Generic_List_Enumerator_T__o *)(auStack_188 + 0x28),
                       Method$System.Collections.Generic.List.Enumerator<Friend>.Dispose());
            return;
          }
          if (local_150.rgctx_data == (Il2CppRGCTXData *)0x0) {
                    /* WARNING: Subroutine does not return */
            FUN_0083c89c();
          }
          bVar5 = System.String$$op_Equality
                            ((System_String_o *)local_150.rgctx_data[2].method,pSVar24,
                             (MethodInfo *)0x0);
        } while (!bVar5);
        System.Collections.Generic.List.Enumerator<object>$$Dispose
                  ((System_Collections_Generic_List_Enumerator_T__o *)(auStack_188 + 0x28),
                   Method$System.Collections.Generic.List.Enumerator<Friend>.Dispose());
      }
    }
    pSVar65 = System.String$$Concat
                        (StringLiteral_1583,pSVar29,StringLiteral_1357,pSVar63,(MethodInfo *)0x0);
    pcVar80 = (chat_log_o *)thunk_FUN_008184f0(chat_log_TypeInfo);
    if (pcVar80 != (chat_log_o *)0x0) {
      chat_log$$.ctor(pcVar80,pSVar65,pSVar63,(UnityEngine_Sprite_o *)0x0,false,
                      (System_Collections_Generic_Dictionary_string__string__o *)0x0,
                      (MethodInfo *)0x0);
      if (DAT_028e6c84 == '\0') {
        FUN_0083c778(&GameServerInterface_TypeInfo);
        DAT_028e6c84 = '\x01';
      }
      pGVar95 = GameServerInterface_TypeInfo->static_fields->Instance;
      if ((pGVar95 != (GameServerInterface_o *)0x0) &&
         (pCVar81 = (pGVar95->fields).game_chat, pCVar81 != (ChatCollection_o *)0x0)) {
        pMVar102 = (MethodInfo *)0x0;
        ChatCollection$$AddLog(pCVar81,pcVar80,(MethodInfo *)0x0);
        if (DAT_028e6c84 == '\0') {
          FUN_0083c778(&GameServerInterface_TypeInfo);
          DAT_028e6c84 = '\x01';
        }
        pGVar95 = GameServerInterface_TypeInfo->static_fields->Instance;
        if (pGVar95 != (GameServerInterface_o *)0x0) {
          GameServerInterface$$GameChatReceived(pGVar95,pcVar80,pMVar102);
          if ((FriendServerReceiver_TypeInfo->_2).cctor_finished == 0) {
            thunk_FUN_008bc8d8();
          }
          pFVar101 = FriendServerReceiver_TypeInfo->static_fields->Instance;
          if (pFVar101 != (FriendServerReceiver_o *)0x0) {
            FriendServerReceiver$$AddToRecentlySeenPlayers
                      (pFVar101,pSVar24,pSVar29,pSVar63,(MethodInfo *)0x0);
            return;
          }
        }
      }
    }
    goto LAB_00958368;
  case 7:
    pSVar24 = Packet$$GetString(incoming,(MethodInfo *)0x0);
    pSVar29 = Packet$$GetString(incoming,(MethodInfo *)0x0);
    uVar6 = Packet$$GetByte(incoming,(MethodInfo *)0x0);
    if (DAT_028e6c84 == '\0') {
      FUN_0083c778(&GameServerInterface_TypeInfo);
      DAT_028e6c84 = '\x01';
    }
    pGVar95 = GameServerInterface_TypeInfo->static_fields->Instance;
    if (pGVar95 != (GameServerInterface_o *)0x0) {
      GameServerInterface$$ShowPlayerLogInOrOut(pGVar95,pSVar29,uVar6,(MethodInfo *)method_06);
      pMVar102 = extraout_x1_04;
      if (uVar6 == 1) {
        if (DAT_028e6c86 == '\0') {
          FUN_0083c778(&GameServerConnector_TypeInfo);
          DAT_028e6c86 = '\x01';
          pMVar102 = extraout_x1_25;
        }
        pGVar90 = GameServerConnector_TypeInfo->static_fields->Instance;
        if (pGVar90 == (GameServerConnector_o *)0x0) goto LAB_00958368;
        (pGVar90->fields).n_others_in_game = (pGVar90->fields).n_others_in_game + 1;
      }
      else if (uVar6 == 0) {
        if (DAT_028e6c86 == '\0') {
          FUN_0083c778(&GameServerConnector_TypeInfo);
          DAT_028e6c86 = '\x01';
          pMVar102 = extraout_x1_05;
        }
        pGVar90 = GameServerConnector_TypeInfo->static_fields->Instance;
        if (pGVar90 == (GameServerConnector_o *)0x0) goto LAB_00958368;
        (pGVar90->fields).n_others_in_game = (pGVar90->fields).n_others_in_game + -1;
        if ((pGVar90->fields).is_host_cached != false) {
          pSVar107 = (__this->fields).unique_ids_given_away;
          if (pSVar107 == (System_Collections_Generic_Dictionary_string__List_int___o *)0x0)
          goto LAB_00958368;
          auVar132 = System.Collections.Generic.Dictionary<>$$ContainsKey
                               (pSVar107,pSVar24,
                                _Method$System.Collections.Generic.Dictionary<string,-List<int>>.Con tainsKey()
                               );
          pMVar102 = auVar132._8_8_;
          if ((auVar132._0_8_ & 1) != 0) {
            pSVar107 = (__this->fields).unique_ids_given_away;
            if (pSVar107 == (System_Collections_Generic_Dictionary_string__List_int___o *)0x0)
            goto LAB_00958368;
            pCVar104 = ConstructionControl_TypeInfo->static_fields->Instance;
            pSVar72 = (System_Collections_Generic_List_int__o *)
                      System.Collections.Generic.Dictionary<>$$get_Item
                                (pSVar107,pSVar24,
                                 _Method$System.Collections.Generic.Dictionary<string,-List<int>>.ge t_Item()
                                );
            if (pCVar104 == (ConstructionControl_o *)0x0) goto LAB_00958368;
            ConstructionControl$$RecycleUniqueIds(pCVar104,pSVar72,(MethodInfo *)0x0);
            pSVar107 = (__this->fields).unique_ids_given_away;
            if (pSVar107 == (System_Collections_Generic_Dictionary_string__List_int___o *)0x0)
            goto LAB_00958368;
            System.Collections.Generic.Dictionary<>$$Remove
                      (pSVar107,pSVar24,
                       _Method$System.Collections.Generic.Dictionary<string,-List<int>>.Remove());
            pMVar102 = extraout_x1_06;
          }
        }
      }
      pFVar83 = FriendServerSender_TypeInfo->static_fields->Instance;
      if (pFVar83 != (FriendServerSender_o *)0x0) {
        FriendServerSender$$UpdateWorldString(pFVar83,pMVar102);
        return;
      }
    }
    goto LAB_00958368;
  case 8:
    pSVar24 = Packet$$GetString(incoming,(MethodInfo *)0x0);
    pSVar29 = Packet$$GetString(incoming,(MethodInfo *)0x0);
    pPVar64 = PlayerData_TypeInfo->static_fields->Instance;
    if (pPVar64 != (PlayerData_o *)0x0) {
      pSVar65 = PlayerData$$GetGlobalString(pPVar64,StringLiteral_14006,(MethodInfo *)0x0);
      bVar5 = System.String$$op_Equality(pSVar24,pSVar65,(MethodInfo *)0x0);
      pSVar63 = _StringLiteral_9503;
      if (!bVar5) {
        pSVar63 = pSVar24;
      }
      bVar5 = System.String$$op_Equality(pSVar29,pSVar65,(MethodInfo *)0x0);
      pSVar24 = _StringLiteral_14261;
      if (!bVar5) {
        pSVar24 = pSVar29;
      }
      pSVar115 = (System_String_array *)FUN_0083c7e4(string[]_TypeInfo,5);
      if (pSVar115 != (System_String_array *)0x0) {
        if (_StringLiteral_1608 == (System_String_o *)0x0) {
          pSVar29 = (System_String_o *)0x0;
        }
        else {
          lVar61 = thunk_FUN_00818404(_StringLiteral_1608,
                                      (((pSVar115->obj).klass)->_1).element_class);
          pSVar29 = _StringLiteral_1608;
          if (lVar61 == 0) goto LAB_00958384;
        }
        if ((int)pSVar115->max_length != 0) {
          pSVar115->m_Items[0] = pSVar29;
          thunk_FUN_008c6b48(pSVar115->m_Items,pSVar29);
          if ((pSVar63 != (System_String_o *)0x0) &&
             (lVar61 = thunk_FUN_00818404(pSVar63,(((pSVar115->obj).klass)->_1).element_class),
             lVar61 == 0)) goto LAB_00958384;
          if (1 < (uint)pSVar115->max_length) {
            pSVar115->m_Items[1] = pSVar63;
            thunk_FUN_008c6b48(pSVar115->m_Items + 1,pSVar63);
            if (_StringLiteral_1437 == (System_String_o *)0x0) {
              pSVar29 = (System_String_o *)0x0;
            }
            else {
              lVar61 = thunk_FUN_00818404(_StringLiteral_1437,
                                          (((pSVar115->obj).klass)->_1).element_class);
              pSVar29 = _StringLiteral_1437;
              if (lVar61 == 0) goto LAB_00958384;
            }
            if (2 < (uint)pSVar115->max_length) {
              pSVar115->m_Items[2] = pSVar29;
              thunk_FUN_008c6b48(pSVar115->m_Items + 2,pSVar29);
              if ((pSVar24 != (System_String_o *)0x0) &&
                 (lVar61 = thunk_FUN_00818404(pSVar24,(((pSVar115->obj).klass)->_1).element_class),
                 lVar61 == 0)) goto LAB_00958384;
              if (3 < (uint)pSVar115->max_length) {
                pSVar115->m_Items[3] = pSVar24;
                thunk_FUN_008c6b48(pSVar115->m_Items + 3,pSVar24);
                if (StringLiteral_1398 == (System_String_o *)0x0) {
                  pSVar24 = (System_String_o *)0x0;
                }
                else {
                  lVar61 = thunk_FUN_00818404(StringLiteral_1398,
                                              (((pSVar115->obj).klass)->_1).element_class);
                  pSVar24 = StringLiteral_1398;
                  if (lVar61 == 0) goto LAB_00958384;
                }
                if (4 < (uint)pSVar115->max_length) {
                  pSVar115->m_Items[4] = pSVar24;
                  thunk_FUN_008c6b48(pSVar115->m_Items + 4,pSVar24);
                  pSVar24 = System.String$$Concat(pSVar115,(MethodInfo *)0x0);
                  if ((CompanionController_TypeInfo->_2).cctor_finished == 0) {
                    thunk_FUN_008bc8d8(CompanionController_TypeInfo);
                  }
                  pCVar49 = CompanionController_TypeInfo->static_fields->Instance;
                  if (pCVar49 != (CompanionController_o *)0x0) {
                    pUVar48 = (pCVar49->fields).companion_died_ico;
                    pcVar80 = (chat_log_o *)thunk_FUN_008184f0(chat_log_TypeInfo);
                    if (pcVar80 != (chat_log_o *)0x0) {
                      chat_log$$.ctor(pcVar80,pSVar24,(System_String_o *)StringLiteral_1.rgctx_data,
                                      pUVar48,false,
                                      (System_Collections_Generic_Dictionary_string__string__o *)0x0
                                      ,(MethodInfo *)0x0);
                      pGVar95 = GameServerInterface_TypeInfo->static_fields->Instance;
                      if ((pGVar95 != (GameServerInterface_o *)0x0) &&
                         (pCVar81 = (pGVar95->fields).game_chat, pCVar81 != (ChatCollection_o *)0x0)
                         ) {
                        pMVar102 = (MethodInfo *)0x0;
                        ChatCollection$$AddLog(pCVar81,pcVar80,(MethodInfo *)0x0);
                        pGVar95 = GameServerInterface_TypeInfo->static_fields->Instance;
                        if (pGVar95 != (GameServerInterface_o *)0x0) {
                          GameServerInterface$$GameChatReceived(pGVar95,pcVar80,pMVar102);
                          return;
                        }
                      }
                    }
                  }
                  goto LAB_00958368;
                }
              }
            }
          }
        }
        goto code_r0x0095836c;
      }
    }
    goto LAB_00958368;
  case 9:
    pSVar24 = Packet$$GetString(incoming,(MethodInfo *)0x0);
    pSVar29 = Packet$$GetString(incoming,(MethodInfo *)0x0);
    if ((CompanionController_TypeInfo->_2).cctor_finished == 0) {
      thunk_FUN_008bc8d8(CompanionController_TypeInfo);
    }
    pCVar49 = CompanionController_TypeInfo->static_fields->Instance;
    if (pCVar49 == (CompanionController_o *)0x0) goto LAB_00958368;
    CompanionController$$OnGuardDie(pCVar49,pSVar24,pSVar29,(MethodInfo *)0x0);
    break;
  case 10:
    pSVar24 = Packet$$GetString(incoming,(MethodInfo *)0x0);
    pSVar29 = Packet$$GetString(incoming,(MethodInfo *)0x0);
    uVar6 = Packet$$GetByte(incoming,(MethodInfo *)0x0);
    pGVar40 = (GameServerReceiver_o *)(ulong)uVar6;
    if (DAT_028e6c81 == '\0') {
      pGVar40 = (GameServerReceiver_o *)FUN_0083c778(&UnityEngine.Vector3_TypeInfo);
      DAT_028e6c81 = '\x01';
    }
    if ((uVar6 & 0xfe) == 2) {
      UVar129 = GameServerReceiver$$UnpackPosition(pGVar40,incoming,(MethodInfo *)method_04);
      fVar128 = UVar129.fields.z;
      fVar127 = UVar129.fields.y;
      fVar125 = UVar129.fields.x;
    }
    else {
      pUVar110 = UnityEngine.Vector3_TypeInfo->static_fields;
      fVar125 = (pUVar110->zeroVector).fields.x;
      fVar127 = (pUVar110->zeroVector).fields.y;
      fVar128 = (pUVar110->zeroVector).fields.z;
    }
    pPVar34 = (Packet_o *)thunk_FUN_008184f0(Packet_TypeInfo);
    if (pPVar34 == (Packet_o *)0x0) goto LAB_00958368;
    Packet$$.ctor(pPVar34,(MethodInfo *)0x0);
    Packet$$PutByte(pPVar34,0xb,(MethodInfo *)0x0);
    Packet$$PutString(pPVar34,pSVar24,(MethodInfo *)0x0);
    Packet$$PutString(pPVar34,pSVar29,(MethodInfo *)0x0);
    pMVar102 = (MethodInfo *)0x0;
    Packet$$PutByte(pPVar34,uVar6,(MethodInfo *)0x0);
    if ((uVar6 & 0xfe) == 2) {
      pGVar106 = extraout_x0_04;
      if (DAT_028e6c85 == '\0') {
        pGVar106 = (GameServerSender_o *)FUN_0083c778(&GameServerSender_TypeInfo);
        DAT_028e6c85 = '\x01';
      }
      if (GameServerSender_TypeInfo->static_fields->Instance == (GameServerSender_o *)0x0)
      goto LAB_00958368;
      pos.fields.y = fVar127;
      pos.fields.x = fVar125;
      pos.fields.z = fVar128;
      GameServerSender$$PackPosition(pGVar106,pPVar34,pos,pMVar102);
    }
    pZVar123 = ZoneDataControl_TypeInfo->static_fields->Instance;
    if ((pZVar123 == (ZoneDataControl_o *)0x0) ||
       (pZVar77 = ZoneDataControl$$LoadZoneDataFromDisk(pZVar123,pSVar29,(MethodInfo *)0x0),
       pZVar77 == (ZoneData_o *)0x0)) goto LAB_00958368;
    ZoneData$$PackForWeb(pZVar77,pPVar34,(MethodInfo *)0x0);
    ZoneData$$ClearOutdoorLandClaims(pZVar77,(MethodInfo *)0x0);
    pZVar123 = ZoneDataControl_TypeInfo->static_fields->Instance;
    if ((pZVar123 == (ZoneDataControl_o *)0x0) ||
       (pSVar27 = ZoneDataControl$$GetZoneTrail(pZVar123,pSVar29,(MethodInfo *)0x0),
       pSVar27 == (System_Collections_Generic_List_string__o *)0x0)) goto LAB_00958368;
    Packet$$PutShort(pPVar34,(float)(pSVar27->fields)._size,(MethodInfo *)0x0);
    System.Collections.Generic.List<object>$$GetEnumerator
              ((System_Collections_Generic_List_Enumerator_T__o *)auStack_188,
               (System_Collections_Generic_List_object__o *)pSVar27,
               Method$System.Collections.Generic.List<string>.GetEnumerator());
    ppIStack_e8 = (Il2CppType **)auStack_188._8_8_;
    local_f0 = (undefined1  [8])auStack_188._0_8_;
    local_e0 = (_union_13)auStack_188._16_8_;
    while (bVar5 = System.Collections.Generic.List.Enumerator<object>$$MoveNext
                             ((System_Collections_Generic_List_Enumerator_T__o *)local_f0,
                              Method$System.Collections.Generic.List.Enumerator<string>.MoveNext()),
          bVar5) {
      Packet$$PutString(pPVar34,(System_String_o *)local_e0.rgctx_data,(MethodInfo *)0x0);
    }
    System.Collections.Generic.List.Enumerator<object>$$Dispose
              ((System_Collections_Generic_List_Enumerator_T__o *)local_f0,
               Method$System.Collections.Generic.List.Enumerator<string>.Dispose());
    pCVar25 = GameServerReceiver$$get_connection(__this_13,method_02);
    goto joined_r0x0095792c;
  case 0xb:
    if (DAT_028e6c85 == '\0') {
      FUN_0083c778(&GameServerSender_TypeInfo);
      DAT_028e6c85 = '\x01';
    }
    pGVar106 = GameServerSender_TypeInfo->static_fields->Instance;
    if (pGVar106 == (GameServerSender_o *)0x0) goto LAB_00958368;
    routine = (pGVar106->fields).zone_data_timeout;
    if (routine != (System_Collections_IEnumerator_o *)0x0) {
      UnityEngine.MonoBehaviour$$StopCoroutine
                ((UnityEngine_MonoBehaviour_o *)pGVar106,routine,(MethodInfo *)0x0);
    }
    uVar6 = Packet$$GetByte(incoming,(MethodInfo *)0x0);
    uVar8 = Packet$$GetByte(incoming,(MethodInfo *)0x0);
    pGVar95 = (GameServerInterface_o *)(ulong)uVar8;
    (__this->fields).waiting_on_initial_zone_data = false;
    if (uVar6 == 1) {
      if (DAT_028e6c84 == '\0') {
        FUN_0083c778(&GameServerInterface_TypeInfo);
        DAT_028e6c84 = '\x01';
      }
      pGVar95 = GameServerInterface_TypeInfo->static_fields->Instance;
      if (pGVar95 == (GameServerInterface_o *)0x0) goto LAB_00958368;
      GameServerInterface$$ProcessIncomingZoneData
                (pGVar95,incoming,uVar8 == 1,(MethodInfo *)method_06);
    }
    else if (uVar6 == 0) {
      if (DAT_028e6c84 == '\0') {
        pGVar95 = (GameServerInterface_o *)FUN_0083c778(&GameServerInterface_TypeInfo);
        DAT_028e6c84 = '\x01';
      }
      if (GameServerInterface_TypeInfo->static_fields->Instance == (GameServerInterface_o *)0x0)
      goto LAB_00958368;
      GameServerInterface$$UnknownZoneGotoSpawn(pGVar95,true,uVar8 == 1,(MethodInfo *)method_06);
    }
    break;
  case 0xc:
    pSVar24 = Packet$$GetString(incoming,(MethodInfo *)0x0);
    pSVar29 = Packet$$GetString(incoming,(MethodInfo *)0x0);
    iVar10 = Packet$$GetShort(incoming,(MethodInfo *)0x0);
    iVar11 = Packet$$GetShort(incoming,(MethodInfo *)0x0);
    if ((ChunkControl_TypeInfo->_2).cctor_finished == 0) {
      thunk_FUN_008bc8d8(ChunkControl_TypeInfo);
    }
    pCVar120 = ChunkControl_TypeInfo->static_fields->Instance;
    if (pCVar120 != (ChunkControl_o *)0x0) {
      pSVar63 = ChunkControl$$GetChunkString
                          (pCVar120,pSVar29,(int)iVar10,(int)iVar11,(MethodInfo *)0x0);
      pCVar120 = ChunkControl_TypeInfo->static_fields->Instance;
      if (pCVar120 != (ChunkControl_o *)0x0) {
        bVar5 = ChunkControl$$IsChunkFullyLoadedOrMidload(pCVar120,pSVar63,(MethodInfo *)0x0);
        if ((ChunkControl_TypeInfo->_2).cctor_finished == 0) {
          thunk_FUN_008bc8d8(ChunkControl_TypeInfo);
        }
        pCVar120 = ChunkControl_TypeInfo->static_fields->Instance;
        if (pCVar120 != (ChunkControl_o *)0x0) {
          if (!bVar5) {
            pCVar45 = ChunkControl$$HostGetChunk
                                (pCVar120,pSVar29,(int)iVar10,(int)iVar11,(MethodInfo *)0x0);
          }
          else {
            pCVar45 = ChunkControl$$GetChunkData(pCVar120,pSVar63,(MethodInfo *)0x0);
          }
          pZVar123 = ZoneDataControl_TypeInfo->static_fields->Instance;
          if (((pZVar123 != (ZoneDataControl_o *)0x0) &&
              (pZVar77 = ZoneDataControl$$LoadZoneDataFromDisk(pZVar123,pSVar29,(MethodInfo *)0x0),
              pZVar77 != (ZoneData_o *)0x0)) && (pCVar45 != (ChunkData_o *)0x0)) {
            pSVar27 = ChunkData$$DetermineBanditCampsWithinChunk
                                (pCVar45,(pZVar77->fields).house_item,(MethodInfo *)0x0);
            pSVar58 = (System_Collections_Generic_List_object__o *)
                      thunk_FUN_008184f0(System.Collections.Generic.List<string>_TypeInfo);
            if ((pSVar58 != (System_Collections_Generic_List_object__o *)0x0) &&
               (System.Collections.Generic.List<object>$$.ctor
                          (pSVar58,Method$System.Collections.Generic.List<string>..ctor()),
               pSVar27 != (System_Collections_Generic_List_string__o *)0x0)) {
              System.Collections.Generic.List<object>$$GetEnumerator
                        ((System_Collections_Generic_List_Enumerator_T__o *)auStack_188,
                         (System_Collections_Generic_List_object__o *)pSVar27,
                         Method$System.Collections.Generic.List<string>.GetEnumerator());
              ppIStack_e8 = (Il2CppType **)auStack_188._8_8_;
              local_f0 = (undefined1  [8])auStack_188._0_8_;
              local_e0 = (_union_13)auStack_188._16_8_;
              while (bVar9 = System.Collections.Generic.List.Enumerator<object>$$MoveNext
                                       ((System_Collections_Generic_List_Enumerator_T__o *)local_f0,
                                        Method$System.Collections.Generic.List.Enumerator<string>.Mo veNext()
                                       ), _Var53 = local_e0, bVar9) {
                pSVar27 = (__this->fields).full_bandit_camps_sent_to_server;
                if (pSVar27 == (System_Collections_Generic_List_string__o *)0x0) {
                    /* WARNING: Subroutine does not return */
                  FUN_0083c89c();
                }
                bVar9 = System.Collections.Generic.List<object>$$Contains
                                  ((System_Collections_Generic_List_object__o *)pSVar27,
                                   (Il2CppObject *)local_e0.rgctx_data,
                                   (MethodInfo_F1A0AC *)
                                   Method$System.Collections.Generic.List<string>.Contains());
                lVar61 = Method$System.Collections.Generic.List<string>.Add();
                if (!bVar9) {
                  pSVar114 = (pSVar58->fields)._items;
                  (pSVar58->fields)._version = (pSVar58->fields)._version + 1;
                  if (pSVar114 == (System_Object_array *)0x0) {
                    /* WARNING: Subroutine does not return */
                    FUN_0083c89c();
                  }
                  uVar18 = (pSVar58->fields)._size;
                  if (uVar18 < (uint)pSVar114->max_length) {
                    (pSVar58->fields)._size = uVar18 + 1;
                    ((_union_13 *)(pSVar114->m_Items + (int)uVar18))->rgctx_data =
                         (Il2CppRGCTXData *)_Var53;
                    thunk_FUN_008c6b48((_union_13 *)(pSVar114->m_Items + (int)uVar18),
                                       _Var53.rgctx_data);
                  }
                  else {
                    (**(code **)(*(long *)(*(long *)(*(long *)(lVar61 + 0x20) + 0xc0) + 0x58) + 8))
                              (pSVar58,_Var53.rgctx_data);
                  }
                  lVar61 = Method$System.Collections.Generic.List<string>.Add();
                  pSVar27 = (__this->fields).full_bandit_camps_sent_to_server;
                  if (pSVar27 == (System_Collections_Generic_List_string__o *)0x0) {
                    /* WARNING: Subroutine does not return */
                    FUN_0083c89c();
                  }
                  pSVar115 = (pSVar27->fields)._items;
                  (pSVar27->fields)._version = (pSVar27->fields)._version + 1;
                  if (pSVar115 == (System_String_array *)0x0) {
                    /* WARNING: Subroutine does not return */
                    FUN_0083c89c();
                  }
                  uVar18 = (pSVar27->fields)._size;
                  if (uVar18 < (uint)pSVar115->max_length) {
                    (pSVar27->fields)._size = uVar18 + 1;
                    ((_union_13 *)(pSVar115->m_Items + (int)uVar18))->rgctx_data =
                         (Il2CppRGCTXData *)_Var53;
                    thunk_FUN_008c6b48((_union_13 *)(pSVar115->m_Items + (int)uVar18),
                                       _Var53.rgctx_data);
                  }
                  else {
                    (**(code **)(*(long *)(*(long *)(*(long *)(lVar61 + 0x20) + 0xc0) + 0x58) + 8))
                              (pSVar27,_Var53.rgctx_data);
                  }
                }
              }
              System.Collections.Generic.List.Enumerator<object>$$Dispose
                        ((System_Collections_Generic_List_Enumerator_T__o *)local_f0,
                         Method$System.Collections.Generic.List.Enumerator<string>.Dispose());
              pPVar34 = (Packet_o *)thunk_FUN_008184f0(Packet_TypeInfo);
              if (pPVar34 != (Packet_o *)0x0) {
                Packet$$.ctor(pPVar34,(MethodInfo *)0x0);
                Packet$$PutByte(pPVar34,0xd,(MethodInfo *)0x0);
                Packet$$PutString(pPVar34,pSVar24,(MethodInfo *)0x0);
                ChunkData$$PackForWeb(pCVar45,pPVar34,(MethodInfo *)0x0);
                Packet$$PutByte(pPVar34,(uint8_t)(pSVar58->fields)._size,(MethodInfo *)0x0);
                System.Collections.Generic.List<object>$$GetEnumerator
                          ((System_Collections_Generic_List_Enumerator_T__o *)auStack_188,pSVar58,
                           Method$System.Collections.Generic.List<string>.GetEnumerator());
                ppIStack_e8 = (Il2CppType **)auStack_188._8_8_;
                local_f0 = (undefined1  [8])auStack_188._0_8_;
                local_e0 = (_union_13)auStack_188._16_8_;
                while (bVar9 = System.Collections.Generic.List.Enumerator<object>$$MoveNext
                                         ((System_Collections_Generic_List_Enumerator_T__o *)
                                          local_f0,
                                          Method$System.Collections.Generic.List.Enumerator<string>. MoveNext()
                                         ), bVar9) {
                  pBVar94 = BanditCampsControl_TypeInfo->static_fields->Instance;
                  if (pBVar94 == (BanditCampsControl_o *)0x0) {
                    /* WARNING: Subroutine does not return */
                    FUN_0083c89c();
                  }
                  pBVar33 = BanditCampsControl$$GetBanditCampInstanceByName
                                      (pBVar94,(System_String_o *)local_e0.rgctx_data,
                                       (MethodInfo *)0x0);
                  if (pBVar33 == (BanditCampInstance_o *)0x0) {
                    /* WARNING: Subroutine does not return */
                    FUN_0083c89c();
                  }
                  BanditCampInstance$$PackForWeb(pBVar33,pPVar34,(MethodInfo *)0x0);
                }
                System.Collections.Generic.List.Enumerator<object>$$Dispose
                          ((System_Collections_Generic_List_Enumerator_T__o *)local_f0,
                           Method$System.Collections.Generic.List.Enumerator<string>.Dispose());
                pCVar25 = GameServerReceiver$$get_connection(__this_12,method_01);
                if (pCVar25 != (Connection_o *)0x0) {
                  Connection$$Send(pCVar25,pPVar34,2,(MethodInfo *)0x0);
                  if (bVar5) {
                    return;
                  }
                  ChunkData$$SaveLandClaimChunkTimersToDisk
                            (pCVar45,pSVar63,(SingleFile_o *)0x0,(MethodInfo *)0x0);
                  return;
                }
              }
            }
          }
        }
      }
    }
    goto LAB_00958368;
  case 0xd:
    pSVar24 = Packet$$GetString(incoming,(MethodInfo *)0x0);
    iVar10 = Packet$$GetShort(incoming,(MethodInfo *)0x0);
    iVar11 = Packet$$GetShort(incoming,(MethodInfo *)0x0);
    if ((ChunkControl_TypeInfo->_2).cctor_finished == 0) {
      thunk_FUN_008bc8d8(ChunkControl_TypeInfo);
    }
    pCVar120 = ChunkControl_TypeInfo->static_fields->Instance;
    if (pCVar120 == (ChunkControl_o *)0x0) goto LAB_00958368;
    pSVar24 = ChunkControl$$GetChunkString
                        (pCVar120,pSVar24,(int)iVar10,(int)iVar11,(MethodInfo *)0x0);
    uVar6 = Packet$$GetByte(incoming,(MethodInfo *)0x0);
    pSVar29 = Packet$$GetString(incoming,(MethodInfo *)0x0);
    if (uVar6 == 1) {
      pGVar106 = GameServerSender_TypeInfo->static_fields->Instance;
      if ((pGVar106 == (GameServerSender_o *)0x0) ||
         (pSVar46 = (pGVar106->fields).cached_chunks,
         pSVar46 == (System_Collections_Generic_Dictionary_string__ChunkData__o *)0x0))
      goto LAB_00958368;
      uVar32 = System.Collections.Generic.Dictionary<>$$ContainsKey
                         (pSVar46,pSVar24,
                          Method$System.Collections.Generic.Dictionary<string,-ChunkData>.ContainsKe y()
                         );
      if ((uVar32 & 1) != 0) {
        pGVar106 = GameServerSender_TypeInfo->static_fields->Instance;
        if (((pGVar106 == (GameServerSender_o *)0x0) ||
            (pSVar46 = (pGVar106->fields).cached_chunks,
            pSVar46 == (System_Collections_Generic_Dictionary_string__ChunkData__o *)0x0)) ||
           (lVar61 = System.Collections.Generic.Dictionary<>$$get_Item
                               (pSVar46,pSVar24,
                                Method$System.Collections.Generic.Dictionary<string,-ChunkData>.get_ Item()
                               ), lVar61 == 0)) goto LAB_00958368;
        bVar5 = System.String$$op_Equality
                          (*(System_String_o **)(lVar61 + 0x58),pSVar29,(MethodInfo *)0x0);
        if (bVar5) {
          pGVar106 = GameServerSender_TypeInfo->static_fields->Instance;
          if ((pGVar106 == (GameServerSender_o *)0x0) ||
             (pSVar46 = (pGVar106->fields).cached_chunks,
             pSVar46 == (System_Collections_Generic_Dictionary_string__ChunkData__o *)0x0))
          goto LAB_00958368;
          pCVar45 = (ChunkData_o *)
                    System.Collections.Generic.Dictionary<>$$get_Item
                              (pSVar46,pSVar24,
                               Method$System.Collections.Generic.Dictionary<string,-ChunkData>.get_I tem()
                              );
          goto LAB_00956c70;
        }
      }
      if ((ChunkControl_TypeInfo->_2).cctor_finished == 0) {
        thunk_FUN_008bc8d8();
      }
      pCVar120 = ChunkControl_TypeInfo->static_fields->Instance;
      if (pCVar120 == (ChunkControl_o *)0x0) goto LAB_00958368;
      ChunkControl$$ChangeChunkStatus(pCVar120,pSVar24,1,(MethodInfo *)0x0);
      pCVar45 = (ChunkData_o *)0x0;
      iVar19 = 6;
    }
    else {
      if (uVar6 == 0) {
        pCVar45 = (ChunkData_o *)thunk_FUN_008184f0(ChunkData_TypeInfo);
        if (pCVar45 == (ChunkData_o *)0x0) goto LAB_00958368;
        ChunkData$$.ctor(pCVar45,(MethodInfo *)0x0);
        ChunkData$$UnpackFromWeb(pCVar45,incoming,(MethodInfo *)0x0);
        ppSVar121 = &(pCVar45->fields).mp_cache_key;
        *ppSVar121 = pSVar29;
        thunk_FUN_008c6b48(ppSVar121,pSVar29);
        if (0x4b < (pCVar45->fields).mp_chunk_size) {
          if (DAT_028e6c85 == '\0') {
            FUN_0083c778(&GameServerSender_TypeInfo);
            DAT_028e6c85 = '\x01';
          }
          pGVar106 = GameServerSender_TypeInfo->static_fields->Instance;
          if ((pGVar106 == (GameServerSender_o *)0x0) ||
             (pSVar46 = (pGVar106->fields).cached_chunks,
             pSVar46 == (System_Collections_Generic_Dictionary_string__ChunkData__o *)0x0))
          goto LAB_00958368;
          uVar32 = System.Collections.Generic.Dictionary<>$$ContainsKey
                             (pSVar46,pSVar24,
                              Method$System.Collections.Generic.Dictionary<string,-ChunkData>.Contai nsKey()
                             );
          if (DAT_028e6c85 == '\0') {
            FUN_0083c778(&GameServerSender_TypeInfo);
            DAT_028e6c85 = '\x01';
          }
          pGVar106 = GameServerSender_TypeInfo->static_fields->Instance;
          if ((pGVar106 == (GameServerSender_o *)0x0) ||
             (pSVar46 = (pGVar106->fields).cached_chunks,
             pSVar46 == (System_Collections_Generic_Dictionary_string__ChunkData__o *)0x0))
          goto LAB_00958368;
          if ((uVar32 & 1) == 0) {
            iVar19 = System.Collections.Generic.Dictionary<>$$get_Count
                               (pSVar46,
                                _Method$System.Collections.Generic.Dictionary<string,-ChunkData>.get _Count()
                               );
            _Var53 = StringLiteral_1;
            if (0x28 < iVar19) {
              if (DAT_028e6c85 == '\0') {
                FUN_0083c778(&GameServerSender_TypeInfo);
                DAT_028e6c85 = '\x01';
              }
              pGVar106 = GameServerSender_TypeInfo->static_fields->Instance;
              if ((pGVar106 == (GameServerSender_o *)0x0) ||
                 (pSVar46 = (pGVar106->fields).cached_chunks,
                 pSVar46 == (System_Collections_Generic_Dictionary_string__ChunkData__o *)0x0))
              goto LAB_00958368;
              System.Collections.Generic.Dictionary<>$$GetEnumerator
                        (auStack_188,pSVar46,
                         _Method$System.Collections.Generic.Dictionary<string,-ChunkData>.GetEnumera tor()
                        );
              ppIStack_138 = (Il2CppType **)auStack_188._8_8_;
              local_140 = (System_Collections_Generic_List_T__o *)auStack_188._0_8_;
              pcStack_128 = (char *)auStack_188._24_8_;
              _Stack_130 = (_union_13)auStack_188._16_8_;
              local_120 = (Il2CppClass *)auStack_188._32_8_;
              dVar126 = 0.0;
              while (uVar32 = System.Collections.Generic.Dictionary.Enumerator<>$$MoveNext
                                        (&local_140,
                                         _Method$System.Collections.Generic.Dictionary.Enumerator<st ring,-ChunkData>.MoveNext()
                                        ), pcVar4 = pcStack_128, _Var3 = _Stack_130,
                    (uVar32 & 1) != 0) {
                if ((System.DateTime_TypeInfo->_2).cctor_finished == 0) {
                  thunk_FUN_008bc8d8();
                }
                SVar84 = System.DateTime$$get_UtcNow((MethodInfo *)0x0);
                if (pcVar4 == (char *)0x0) {
                    /* WARNING: Subroutine does not return */
                  FUN_0083c89c();
                }
                _Stack_148 = (_union_14)
                             System.DateTime$$op_Subtraction
                                       (SVar84,(System_DateTime_o)
                                               ((System_DateTime_Fields *)(pcVar4 + 0x68))->
                                               _dateData,(MethodInfo *)0x0);
                if ((System.TimeSpan_TypeInfo->_2).cctor_finished == 0) {
                  thunk_FUN_008bc8d8();
                }
                dVar124 = System.TimeSpan$$get_TotalSeconds
                                    ((System_TimeSpan_o)&_Stack_148,(MethodInfo *)0x0);
                if (dVar126 < dVar124) {
                  _Var53 = _Var3;
                  dVar126 = dVar124;
                }
              }
              System.Collections.Generic.Dictionary.Enumerator<>$$Dispose
                        (&local_140,
                         _Method$System.Collections.Generic.Dictionary.Enumerator<string,-ChunkData> .Dispose()
                        );
              bVar5 = System.String$$op_Inequality
                                ((System_String_o *)_Var53.rgctx_data,
                                 (System_String_o *)StringLiteral_1.rgctx_data,(MethodInfo *)0x0);
              if (bVar5) {
                if (DAT_028e6c85 == '\0') {
                  FUN_0083c778(&GameServerSender_TypeInfo);
                  DAT_028e6c85 = '\x01';
                }
                pGVar106 = GameServerSender_TypeInfo->static_fields->Instance;
                if ((pGVar106 == (GameServerSender_o *)0x0) ||
                   (pSVar46 = (pGVar106->fields).cached_chunks,
                   pSVar46 == (System_Collections_Generic_Dictionary_string__ChunkData__o *)0x0))
                goto LAB_00958368;
                System.Collections.Generic.Dictionary<>$$Remove
                          (pSVar46,_Var53.rgctx_data,
                           Method$System.Collections.Generic.Dictionary<string,-ChunkData>.Remove())
                ;
              }
            }
            if (DAT_028e6c85 == '\0') {
              FUN_0083c778(&GameServerSender_TypeInfo);
              DAT_028e6c85 = '\x01';
            }
            pGVar106 = GameServerSender_TypeInfo->static_fields->Instance;
            if ((pGVar106 == (GameServerSender_o *)0x0) ||
               (pSVar46 = (pGVar106->fields).cached_chunks,
               pSVar46 == (System_Collections_Generic_Dictionary_string__ChunkData__o *)0x0))
            goto LAB_00958368;
            System.Collections.Generic.Dictionary<>$$Add
                      (pSVar46,pSVar24,pCVar45,
                       Method$System.Collections.Generic.Dictionary<string,-ChunkData>.Add());
          }
          else {
            System.Collections.Generic.Dictionary<>$$set_Item
                      (pSVar46,pSVar24,pCVar45,
                       _Method$System.Collections.Generic.Dictionary<string,-ChunkData>.set_Item());
          }
        }
      }
      else {
        pCVar45 = (ChunkData_o *)0x0;
      }
LAB_00956c70:
      iVar19 = 0;
    }
    iVar10 = Packet$$GetShort(incoming,(MethodInfo *)0x0);
    iVar20 = (int)iVar10;
    if (0 < iVar20) {
      do {
        pBVar33 = BanditCampInstance$$UnpackFromWeb(incoming,(MethodInfo *)0x0);
        pBVar94 = BanditCampsControl_TypeInfo->static_fields->Instance;
        if (((pBVar94 == (BanditCampsControl_o *)0x0) || (pBVar33 == (BanditCampInstance_o *)0x0))
           || (pSVar31 = (pBVar94->fields).loaded_bandit_camp_instances,
              pSVar31 == (System_Collections_Generic_Dictionary_string__BanditCampInstance__o *)0x0)
           ) goto LAB_00958368;
        uVar32 = System.Collections.Generic.Dictionary<>$$ContainsKey
                           (pSVar31,(pBVar33->fields).instance_name,
                            Method$System.Collections.Generic.Dictionary<string,-BanditCampInstance> .ContainsKey()
                           );
        if ((uVar32 & 1) == 0) {
          pBVar94 = BanditCampsControl_TypeInfo->static_fields->Instance;
          if ((pBVar94 == (BanditCampsControl_o *)0x0) ||
             (pSVar31 = (pBVar94->fields).loaded_bandit_camp_instances,
             pSVar31 == (System_Collections_Generic_Dictionary_string__BanditCampInstance__o *)0x0))
          goto LAB_00958368;
          System.Collections.Generic.Dictionary<>$$Add
                    (pSVar31,(pBVar33->fields).instance_name,pBVar33,
                     Method$System.Collections.Generic.Dictionary<string,-BanditCampInstance>.Add())
          ;
        }
        iVar20 = iVar20 + -1;
      } while (iVar20 != 0);
    }
    if (DAT_028e6c85 == '\0') {
      FUN_0083c778(&GameServerSender_TypeInfo);
      DAT_028e6c85 = '\x01';
    }
    pGVar106 = GameServerSender_TypeInfo->static_fields->Instance;
    if ((pGVar106 != (GameServerSender_o *)0x0) &&
       (pSVar27 = (pGVar106->fields).chunks_mid_request,
       pSVar27 != (System_Collections_Generic_List_string__o *)0x0)) {
      System.Collections.Generic.List<object>$$Remove
                ((System_Collections_Generic_List_object__o *)pSVar27,(Il2CppObject *)pSVar24,
                 (MethodInfo_F1AFDC *)Method$System.Collections.Generic.List<string>.Remove());
      if (iVar19 != 0) {
        return;
      }
      if ((ChunkControl_TypeInfo->_2).cctor_finished == 0) {
        thunk_FUN_008bc8d8();
      }
      pCVar120 = ChunkControl_TypeInfo->static_fields->Instance;
      if (pCVar120 != (ChunkControl_o *)0x0) {
        bVar5 = ChunkControl$$ChunkExists(pCVar120,pSVar24,(MethodInfo *)0x0);
        if (!bVar5) {
          return;
        }
        if ((System.DateTime_TypeInfo->_2).cctor_finished == 0) {
          thunk_FUN_008bc8d8();
        }
        SVar84 = System.DateTime$$get_UtcNow((MethodInfo *)0x0);
        if (pCVar45 != (ChunkData_o *)0x0) {
          (pCVar45->fields).mp_cache_last_used.fields._dateData = (uint64_t)SVar84.fields._dateData;
          if ((ChunkControl_TypeInfo->_2).cctor_finished == 0) {
            thunk_FUN_008bc8d8();
          }
          pCVar120 = ChunkControl_TypeInfo->static_fields->Instance;
          if ((pCVar120 != (ChunkControl_o *)0x0) &&
             (pCVar47 = ChunkControl$$GetChunk(pCVar120,pSVar24,(MethodInfo *)0x0),
             pCVar47 != (Chunk_o *)0x0)) {
            ppCVar85 = &(pCVar47->fields).chunk_data;
            *ppCVar85 = pCVar45;
            thunk_FUN_008c6b48(ppCVar85,pCVar45);
            pCVar120 = ChunkControl_TypeInfo->static_fields->Instance;
            if (pCVar120 != (ChunkControl_o *)0x0) {
              ChunkControl$$ChangeChunkStatus(pCVar120,pSVar24,3,(MethodInfo *)0x0);
              return;
            }
          }
        }
      }
    }
    goto LAB_00958368;
  case 0x11:
    pGVar40 = (GameServerReceiver_o *)Packet$$GetString(incoming,(MethodInfo *)0x0);
    UVar129 = GameServerReceiver$$UnpackPosition(pGVar40,incoming,(MethodInfo *)method_04);
    UVar130 = GameServerReceiver$$UnpackPosition(__this_05,incoming,(MethodInfo *)method_04);
    UVar131 = GameServerReceiver$$UnpackRotation(__this_06,incoming,(MethodInfo *)method_04);
    if (DAT_028e6c84 == '\0') {
      FUN_0083c778(&GameServerInterface_TypeInfo);
      DAT_028e6c84 = '\x01';
    }
    pGVar95 = GameServerInterface_TypeInfo->static_fields->Instance;
    if ((pGVar95 != (GameServerInterface_o *)0x0) &&
       (pSVar39 = (pGVar95->fields).nearby_players,
       pSVar39 != (System_Collections_Generic_Dictionary_string__OnlinePlayer__o *)0x0)) {
      uVar32 = System.Collections.Generic.Dictionary<>$$ContainsKey
                         (pSVar39,pGVar40,
                          Method$System.Collections.Generic.Dictionary<string,-OnlinePlayer>.Contain sKey()
                         );
      if ((uVar32 & 1) == 0) {
        return;
      }
      if (DAT_028e6c84 == '\0') {
        FUN_0083c778(&GameServerInterface_TypeInfo);
        DAT_028e6c84 = '\x01';
      }
      pGVar95 = GameServerInterface_TypeInfo->static_fields->Instance;
      if (((pGVar95 != (GameServerInterface_o *)0x0) &&
          (pSVar39 = (pGVar95->fields).nearby_players,
          pSVar39 != (System_Collections_Generic_Dictionary_string__OnlinePlayer__o *)0x0)) &&
         (lVar61 = System.Collections.Generic.Dictionary<>$$get_Item
                             (pSVar39,pGVar40,
                              Method$System.Collections.Generic.Dictionary<string,-OnlinePlayer>.get _Item()
                             ), lVar61 != 0)) {
        pUVar74 = *(UnityEngine_Object_o **)(lVar61 + 0x10);
        if ((UnityEngine.Object_TypeInfo->_2).cctor_finished == 0) {
          thunk_FUN_008bc8d8();
        }
        bVar5 = UnityEngine.Object$$op_Equality
                          (pUVar74,(UnityEngine_Object_o *)0x0,(MethodInfo *)0x0);
        if (bVar5) {
          return;
        }
        if ((*(UnityEngine_GameObject_o **)(lVar61 + 0x10) != (UnityEngine_GameObject_o *)0x0) &&
           (pIVar57 = UnityEngine.GameObject$$GetComponent<object>
                                (*(UnityEngine_GameObject_o **)(lVar61 + 0x10),
                                 Method$UnityEngine.GameObject.GetComponent<SharedCreature>()),
           pIVar57 != (Il2CppObject *)0x0)) {
          if (*(char *)((long)&pIVar57[6].monitor + 4) != '\0') {
            return;
          }
          if ((*(UnityEngine_GameObject_o **)(lVar61 + 0x10) != (UnityEngine_GameObject_o *)0x0) &&
             (pIVar57 = UnityEngine.GameObject$$GetComponent<object>
                                  (*(UnityEngine_GameObject_o **)(lVar61 + 0x10),
                                   Method$UnityEngine.GameObject.GetComponent<SharedCreature>()),
             pIVar57 != (Il2CppObject *)0x0)) {
            pIVar54 = pIVar57[4].klass;
            if ((UnityEngine.Object_TypeInfo->_2).cctor_finished == 0) {
              thunk_FUN_008bc8d8(UnityEngine.Object_TypeInfo);
            }
            pMVar102 = (MethodInfo *)0x0;
            bVar5 = UnityEngine.Object$$op_Inequality
                              ((UnityEngine_Object_o *)pIVar54,(UnityEngine_Object_o *)0x0,
                               (MethodInfo *)0x0);
            if (bVar5) {
              return;
            }
            iVar19 = *(int *)(lVar61 + 0x2c);
            if (iVar19 == 0) {
              pGVar95 = (GameServerInterface_o *)0x0;
              if (DAT_028e6c84 == '\0') {
                pGVar95 = (GameServerInterface_o *)FUN_0083c778(&GameServerInterface_TypeInfo);
                DAT_028e6c84 = '\x01';
              }
              if (GameServerInterface_TypeInfo->static_fields->Instance ==
                  (GameServerInterface_o *)0x0) goto LAB_00958368;
              GameServerInterface$$CreateMovementSmoother
                        (pGVar95,*(UnityEngine_GameObject_o **)(lVar61 + 0x10),UVar129,UVar130,
                         pMVar102);
              iVar19 = 3;
              *(undefined4 *)(lVar61 + 0x2c) = 3;
            }
            *(int *)(lVar61 + 0x2c) = iVar19 + -1;
            if ((*(UnityEngine_GameObject_o **)(lVar61 + 0x10) != (UnityEngine_GameObject_o *)0x0)
               && (pSVar26 = (SharedCreature_o *)
                             UnityEngine.GameObject$$GetComponent<object>
                                       (*(UnityEngine_GameObject_o **)(lVar61 + 0x10),
                                        Method$UnityEngine.GameObject.GetComponent<SharedCreature>()
                                       ), pSVar26 != (SharedCreature_o *)0x0)) {
              SharedCreature$$SetMoveTo(pSVar26,UVar130,(MethodInfo *)0x0);
              if ((*(UnityEngine_GameObject_o **)(lVar61 + 0x10) != (UnityEngine_GameObject_o *)0x0)
                 && (pSVar26 = (SharedCreature_o *)
                               UnityEngine.GameObject$$GetComponent<object>
                                         (*(UnityEngine_GameObject_o **)(lVar61 + 0x10),
                                          Method$UnityEngine.GameObject.GetComponent<SharedCreature> ()
                                         ), pSVar26 != (SharedCreature_o *)0x0)) {
                SharedCreature$$SnapSpotterRotation(pSVar26,UVar131,(MethodInfo *)0x0);
                return;
              }
            }
          }
        }
      }
    }
    goto LAB_00958368;
  case 0x12:
    iVar10 = Packet$$GetShort(incoming,(MethodInfo *)0x0);
    iVar19 = (int)iVar10;
    if (0 < iVar19) {
      do {
        if (DAT_028e6c84 == '\0') {
          FUN_0083c778(&GameServerInterface_TypeInfo);
          DAT_028e6c84 = '\x01';
        }
        pGVar95 = GameServerInterface_TypeInfo->static_fields->Instance;
        if (pGVar95 == (GameServerInterface_o *)0x0) goto LAB_00958368;
        GameServerInterface$$NewPlayerNearby(pGVar95,incoming,(MethodInfo *)method_04);
        iVar19 = iVar19 + -1;
      } while (iVar19 != 0);
    }
    iVar10 = Packet$$GetShort(incoming,(MethodInfo *)0x0);
    iVar19 = (int)iVar10;
    if (0 < iVar19) {
      do {
        if (DAT_028e6c84 == '\0') {
          FUN_0083c778(&GameServerInterface_TypeInfo);
          DAT_028e6c84 = '\x01';
        }
        pGVar95 = GameServerInterface_TypeInfo->static_fields->Instance;
        if (pGVar95 == (GameServerInterface_o *)0x0) goto LAB_00958368;
        GameServerInterface$$NearbyPlayerWentAway(pGVar95,incoming,(MethodInfo *)method_04);
        iVar19 = iVar19 + -1;
      } while (iVar19 != 0);
    }
    break;
  case 0x13:
    uVar6 = Packet$$GetByte(incoming,(MethodInfo *)0x0);
    if (uVar6 == 0) {
      if (DAT_028e6c84 == '\0') {
        FUN_0083c778(&GameServerInterface_TypeInfo);
        DAT_028e6c84 = '\x01';
      }
      pGVar95 = GameServerInterface_TypeInfo->static_fields->Instance;
      if (pGVar95 == (GameServerInterface_o *)0x0) goto LAB_00958368;
      GameServerInterface$$NearbyPlayerWentAway(pGVar95,incoming,(MethodInfo *)method_04);
    }
    else if (uVar6 == 1) {
      if (DAT_028e6c84 == '\0') {
        FUN_0083c778(&GameServerInterface_TypeInfo);
        DAT_028e6c84 = '\x01';
      }
      pGVar95 = GameServerInterface_TypeInfo->static_fields->Instance;
      if (pGVar95 == (GameServerInterface_o *)0x0) goto LAB_00958368;
      GameServerInterface$$NewPlayerNearby(pGVar95,incoming,(MethodInfo *)method_04);
    }
    break;
  case 0x15:
    pSVar24 = Packet$$GetString(incoming,(MethodInfo *)0x0);
    if (DAT_028e6c84 == '\0') {
      FUN_0083c778(&GameServerInterface_TypeInfo);
      DAT_028e6c84 = '\x01';
    }
    pGVar95 = GameServerInterface_TypeInfo->static_fields->Instance;
    if (pGVar95 == (GameServerInterface_o *)0x0) goto LAB_00958368;
    GameServerInterface$$StartTeleportPlayer(pGVar95,pSVar24,(MethodInfo *)method_04);
    break;
  case 0x16:
    pGVar40 = (GameServerReceiver_o *)Packet$$GetString(incoming,(MethodInfo *)0x0);
    UVar129 = GameServerReceiver$$UnpackPosition(pGVar40,incoming,(MethodInfo *)method_04);
    if (DAT_028e6c84 == '\0') {
      FUN_0083c778(&GameServerInterface_TypeInfo);
      DAT_028e6c84 = '\x01';
    }
    pGVar95 = GameServerInterface_TypeInfo->static_fields->Instance;
    if (pGVar95 == (GameServerInterface_o *)0x0) goto LAB_00958368;
    GameServerInterface$$EndTeleportPlayer
              (pGVar95,(System_String_o *)pGVar40,UVar129,(MethodInfo *)method_04);
    break;
  case 0x17:
    GameServerReceiver$$ReceiveDaynight(__this,incoming,(MethodInfo *)&Startup_TypeInfo);
    break;
  case 0x18:
    pSVar24 = Packet$$GetString(incoming,(MethodInfo *)0x0);
    uVar6 = Packet$$GetByte(incoming,(MethodInfo *)0x0);
    pIVar68 = InventoryItem$$UnpackFromWeb(incoming,(MethodInfo *)0x0);
    if (DAT_028e6c84 == '\0') {
      FUN_0083c778(&GameServerInterface_TypeInfo);
      DAT_028e6c84 = '\x01';
    }
    pGVar95 = GameServerInterface_TypeInfo->static_fields->Instance;
    if (pGVar95 == (GameServerInterface_o *)0x0) goto LAB_00958368;
    GameServerInterface$$PlayerChangeEquip(pGVar95,pSVar24,uVar6,pIVar68,(MethodInfo *)method_07);
    break;
  case 0x19:
    pSVar24 = Packet$$GetString(incoming,(MethodInfo *)0x0);
    iVar10 = Packet$$GetShort(incoming,(MethodInfo *)0x0);
    pSVar27 = (System_Collections_Generic_List_string__o *)
              thunk_FUN_008184f0(System.Collections.Generic.List<string>_TypeInfo);
    if (pSVar27 != (System_Collections_Generic_List_string__o *)0x0) {
      iVar19 = (int)iVar10;
      System.Collections.Generic.List<object>$$.ctor
                ((System_Collections_Generic_List_object__o *)pSVar27,
                 Method$System.Collections.Generic.List<string>..ctor());
      if (0 < iVar19) {
        do {
          pSVar29 = Packet$$GetString(incoming,(MethodInfo *)0x0);
          lVar61 = Method$System.Collections.Generic.List<string>.Add();
          pSVar115 = (pSVar27->fields)._items;
          (pSVar27->fields)._version = (pSVar27->fields)._version + 1;
          if (pSVar115 == (System_String_array *)0x0) goto LAB_00958368;
          uVar18 = (pSVar27->fields)._size;
          if (uVar18 < (uint)pSVar115->max_length) {
            (pSVar27->fields)._size = uVar18 + 1;
            pSVar115->m_Items[(int)uVar18] = pSVar29;
            thunk_FUN_008c6b48();
          }
          else {
            (**(code **)(*(long *)(*(long *)(*(long *)(lVar61 + 0x20) + 0xc0) + 0x58) + 8))(pSVar27)
            ;
          }
          iVar19 = iVar19 + -1;
        } while (iVar19 != 0);
      }
      if (DAT_028e6c84 == '\0') {
        FUN_0083c778(&GameServerInterface_TypeInfo);
        DAT_028e6c84 = '\x01';
      }
      pGVar95 = GameServerInterface_TypeInfo->static_fields->Instance;
      if (pGVar95 != (GameServerInterface_o *)0x0) {
        GameServerInterface$$OtherPlayerChangeCreatures
                  (pGVar95,pSVar24,pSVar27,(MethodInfo *)method_06);
        return;
      }
    }
    goto LAB_00958368;
  case 0x1b:
    pPVar52 = PopupControl_TypeInfo->static_fields->Instance;
    if (pPVar52 == (PopupControl_o *)0x0) goto LAB_00958368;
    PopupControl$$HideAll(pPVar52,(MethodInfo *)0x0);
    iVar17 = Packet$$GetLong(incoming,(MethodInfo *)0x0);
    pBVar41 = (BasketContents_o *)thunk_FUN_008184f0(BasketContents_TypeInfo);
    if (pBVar41 == (BasketContents_o *)0x0) goto LAB_00958368;
    BasketContents$$.ctor(pBVar41,incoming,(MethodInfo *)0x0);
    if ((inventory_ctr_TypeInfo->_2).cctor_finished == 0) {
      thunk_FUN_008bc8d8();
    }
    piVar112 = inventory_ctr_TypeInfo->static_fields->Instance;
    goto joined_r0x00951adc;
  case 0x1c:
    pSVar24 = Packet$$GetString(incoming,(MethodInfo *)0x0);
    iVar17 = Packet$$GetLong(incoming,(MethodInfo *)0x0);
    pBVar41 = (BasketContents_o *)thunk_FUN_008184f0(BasketContents_TypeInfo);
    if (pBVar41 == (BasketContents_o *)0x0) goto LAB_00958368;
    BasketContents$$.ctor(pBVar41,(MethodInfo *)0x0);
    BasketContents$$LoadFromDiskAsContainer(pBVar41,iVar17,(MethodInfo *)0x0);
    pPVar34 = (Packet_o *)thunk_FUN_008184f0(Packet_TypeInfo);
    if (pPVar34 == (Packet_o *)0x0) goto LAB_00958368;
    Packet$$.ctor(pPVar34,(MethodInfo *)0x0);
    Packet$$PutByte(pPVar34,0x1b,(MethodInfo *)0x0);
    Packet$$PutString(pPVar34,pSVar24,(MethodInfo *)0x0);
    Packet$$PutLong(pPVar34,iVar17,(MethodInfo *)0x0);
    BasketContents$$Pack(pBVar41,pPVar34,(MethodInfo *)0x0);
    pCVar25 = GameServerReceiver$$get_connection(__this_03,method_00);
    if (pCVar25 == (Connection_o *)0x0) goto LAB_00958368;
    iVar17 = 2;
    goto code_r0x00957938;
  case 0x1d:
    pPVar52 = PopupControl_TypeInfo->static_fields->Instance;
    if (pPVar52 == (PopupControl_o *)0x0) goto LAB_00958368;
    PopupControl$$HideAll(pPVar52,(MethodInfo *)0x0);
    iVar17 = Packet$$GetLong(incoming,(MethodInfo *)0x0);
    __this_17 = LootControl_TypeInfo->static_fields->Instance;
    if ((GameController_TypeInfo->_2).cctor_finished == 0) {
      thunk_FUN_008bc8d8(GameController_TypeInfo);
    }
    pGVar93 = GameController_TypeInfo->static_fields->Instance;
    if ((pGVar93 == (GameController_o *)0x0) || (__this_17 == (LootControl_o *)0x0))
    goto LAB_00958368;
    pBVar41 = LootControl$$GenerateLootChest
                        (__this_17,(pGVar93->fields).interacting_element_item,(MethodInfo *)0x0);
    if ((inventory_ctr_TypeInfo->_2).cctor_finished == 0) {
      thunk_FUN_008bc8d8(inventory_ctr_TypeInfo);
    }
    piVar112 = inventory_ctr_TypeInfo->static_fields->Instance;
    if (piVar112 == (inventory_ctr_o *)0x0) goto LAB_00958368;
    inventory_ctr$$AddChestRespawnAndRedraw(piVar112,(MethodInfo *)0x0);
    piVar112 = inventory_ctr_TypeInfo->static_fields->Instance;
joined_r0x00951adc:
    if (piVar112 == (inventory_ctr_o *)0x0) goto LAB_00958368;
    inventory_ctr$$SucceedOpenWorldContainer(piVar112,iVar17,pBVar41,(MethodInfo *)0x0);
    break;
  case 0x1e:
    iVar17 = Packet$$GetLong(incoming,(MethodInfo *)0x0);
    pBVar41 = (BasketContents_o *)thunk_FUN_008184f0(BasketContents_TypeInfo);
    if (pBVar41 == (BasketContents_o *)0x0) goto LAB_00958368;
    BasketContents$$.ctor(pBVar41,incoming,(MethodInfo *)0x0);
    Packet$$GetString(incoming,(MethodInfo *)0x0);
    BasketContents$$SaveToAllAsContainer(pBVar41,iVar17,(MethodInfo *)0x0);
    break;
  case 0x20:
    pIVar68 = InventoryItem$$UnpackFromWeb(incoming,(MethodInfo *)0x0);
    uVar6 = Packet$$GetByte(incoming,(MethodInfo *)0x0);
    pSVar24 = Packet$$GetString(incoming,(MethodInfo *)0x0);
    iVar10 = Packet$$GetShort(incoming,(MethodInfo *)0x0);
    iVar11 = Packet$$GetShort(incoming,(MethodInfo *)0x0);
    iVar12 = Packet$$GetShort(incoming,(MethodInfo *)0x0);
    iVar13 = Packet$$GetShort(incoming,(MethodInfo *)0x0);
    pSVar29 = Packet$$GetString(incoming,(MethodInfo *)0x0);
    pSVar63 = Packet$$GetString(incoming,(MethodInfo *)0x0);
    if ((ChunkControl_TypeInfo->_2).cctor_finished == 0) {
      thunk_FUN_008bc8d8(ChunkControl_TypeInfo);
    }
    pCVar120 = ChunkControl_TypeInfo->static_fields->Instance;
    if (pCVar120 != (ChunkControl_o *)0x0) {
      iVar19 = (int)iVar10;
      iVar20 = (int)iVar11;
      pSVar65 = ChunkControl$$GetChunkString(pCVar120,pSVar24,iVar19,iVar20,(MethodInfo *)0x0);
      pCVar120 = ChunkControl_TypeInfo->static_fields->Instance;
      if (pCVar120 != (ChunkControl_o *)0x0) {
        bVar5 = ChunkControl$$IsChunkFullyLoadedOrMidload(pCVar120,pSVar65,(MethodInfo *)0x0);
        if (bVar5) {
          if ((ChunkControl_TypeInfo->_2).cctor_finished == 0) {
            thunk_FUN_008bc8d8();
          }
          pCVar120 = ChunkControl_TypeInfo->static_fields->Instance;
          if ((pCVar120 == (ChunkControl_o *)0x0) ||
             (pCVar47 = ChunkControl$$GetChunk(pCVar120,pSVar65,(MethodInfo *)0x0),
             pCVar47 == (Chunk_o *)0x0)) goto LAB_00958368;
        }
        else {
          if (DAT_028e6c86 == '\0') {
            FUN_0083c778(&GameServerConnector_TypeInfo);
            DAT_028e6c86 = '\x01';
          }
          pGVar90 = GameServerConnector_TypeInfo->static_fields->Instance;
          if (pGVar90 == (GameServerConnector_o *)0x0) goto LAB_00958368;
          if ((pGVar90->fields).is_host_cached == false) {
            if (pIVar68 != (InventoryItem_o *)0x0) {
              pSVar63 = InventoryItem$$get_item_name(pIVar68,(MethodInfo *)0x0);
              bVar5 = System.String$$op_Equality(pSVar63,StringLiteral_1311,(MethodInfo *)0x0);
              if (!bVar5) {
                pSVar63 = InventoryItem$$get_item_name(pIVar68,(MethodInfo *)0x0);
                bVar5 = System.String$$op_Equality(pSVar63,StringLiteral_1338,(MethodInfo *)0x0);
                if (!bVar5) {
                  pSVar63 = InventoryItem$$get_item_name(pIVar68,(MethodInfo *)0x0);
                  bVar5 = System.String$$op_Equality(pSVar63,StringLiteral_1934,(MethodInfo *)0x0);
                  if (!bVar5) {
                    return;
                  }
                }
              }
              if ((LandClaimControl_TypeInfo->_2).cctor_finished == 0) {
                thunk_FUN_008bc8d8();
              }
              pLVar67 = LandClaimControl_TypeInfo->static_fields->Instance;
              if (pLVar67 != (LandClaimControl_o *)0x0) {
                LandClaimControl$$AddLandClaimsToNearbyChunks
                          (pLVar67,pSVar24,iVar19,iVar20,(int)iVar12,(int)iVar13,pIVar68,pSVar29,
                           (MethodInfo *)0x0);
                return;
              }
            }
            goto LAB_00958368;
          }
        }
        pCVar104 = ConstructionControl_TypeInfo->static_fields->Instance;
        if (pCVar104 != (ConstructionControl_o *)0x0) {
          ConstructionControl$$PlayerBuildAt
                    (pCVar104,pIVar68,pSVar24,iVar19,iVar20,(int)iVar12,(int)iVar13,(uint)uVar6,2,
                     pSVar29,pSVar63,(MethodInfo *)0x0);
          return;
        }
      }
    }
    goto LAB_00958368;
  case 0x21:
    pSVar24 = Packet$$GetString(incoming,(MethodInfo *)0x0);
    iVar10 = Packet$$GetShort(incoming,(MethodInfo *)0x0);
    iVar11 = Packet$$GetShort(incoming,(MethodInfo *)0x0);
    iVar12 = Packet$$GetShort(incoming,(MethodInfo *)0x0);
    iVar13 = Packet$$GetShort(incoming,(MethodInfo *)0x0);
    uVar6 = Packet$$GetByte(incoming,(MethodInfo *)0x0);
    pIVar68 = InventoryItem$$UnpackFromWeb(incoming,(MethodInfo *)0x0);
    __this_04 = (ChunkElement_o *)thunk_FUN_008184f0(ChunkElement_TypeInfo);
    if (__this_04 != (ChunkElement_o *)0x0) {
      ChunkElement$$.ctor(__this_04,pIVar68,(uint)uVar6,(MethodInfo *)0x0);
      pSVar29 = Packet$$GetString(incoming,(MethodInfo *)0x0);
      if ((ChunkControl_TypeInfo->_2).cctor_finished == 0) {
        thunk_FUN_008bc8d8(ChunkControl_TypeInfo);
      }
      pCVar120 = ChunkControl_TypeInfo->static_fields->Instance;
      if (pCVar120 != (ChunkControl_o *)0x0) {
        iVar19 = (int)iVar10;
        iVar20 = (int)iVar11;
        pSVar63 = ChunkControl$$GetChunkString(pCVar120,pSVar24,iVar19,iVar20,(MethodInfo *)0x0);
        pCVar120 = ChunkControl_TypeInfo->static_fields->Instance;
        if (pCVar120 != (ChunkControl_o *)0x0) {
          bVar5 = ChunkControl$$IsChunkFullyLoadedOrMidload(pCVar120,pSVar63,(MethodInfo *)0x0);
          if (!bVar5) {
            if (DAT_028e6c86 == '\0') {
              FUN_0083c778(&GameServerConnector_TypeInfo);
              DAT_028e6c86 = '\x01';
            }
            pGVar90 = GameServerConnector_TypeInfo->static_fields->Instance;
            if (pGVar90 == (GameServerConnector_o *)0x0) goto LAB_00958368;
            if ((pGVar90->fields).is_host_cached == false) {
              if (pIVar68 != (InventoryItem_o *)0x0) {
                pSVar29 = InventoryItem$$get_item_name(pIVar68,(MethodInfo *)0x0);
                bVar5 = System.String$$op_Equality(pSVar29,StringLiteral_1311,(MethodInfo *)0x0);
                if (!bVar5) {
                  pSVar29 = InventoryItem$$get_item_name(pIVar68,(MethodInfo *)0x0);
                  bVar5 = System.String$$op_Equality(pSVar29,StringLiteral_1338,(MethodInfo *)0x0);
                  if (!bVar5) {
                    pSVar29 = InventoryItem$$get_item_name(pIVar68,(MethodInfo *)0x0);
                    bVar5 = System.String$$op_Equality(pSVar29,StringLiteral_1934,(MethodInfo *)0x0)
                    ;
                    if (!bVar5) {
                      return;
                    }
                  }
                }
                if ((LandClaimControl_TypeInfo->_2).cctor_finished == 0) {
                  thunk_FUN_008bc8d8();
                }
                pLVar67 = LandClaimControl_TypeInfo->static_fields->Instance;
                if (pLVar67 != (LandClaimControl_o *)0x0) {
                  LandClaimControl$$RemoveLandClaimsFromNearbyChunks
                            (pLVar67,pSVar24,iVar19,iVar20,(int)iVar12,(int)iVar13,(MethodInfo *)0x0
                            );
                  return;
                }
              }
              goto LAB_00958368;
            }
          }
          pCVar104 = ConstructionControl_TypeInfo->static_fields->Instance;
          if (pCVar104 != (ConstructionControl_o *)0x0) {
            ConstructionControl$$PlayerRemoveAt
                      (pCVar104,__this_04,pSVar24,iVar19,iVar20,(int)iVar12,(int)iVar13,1,pSVar29,
                       (MethodInfo *)0x0);
            return;
          }
        }
      }
    }
    goto LAB_00958368;
  case 0x22:
    pIVar68 = InventoryItem$$UnpackFromWeb(incoming,(MethodInfo *)0x0);
    pIVar69 = InventoryItem$$UnpackFromWeb(incoming,(MethodInfo *)0x0);
    uVar6 = Packet$$GetByte(incoming,(MethodInfo *)0x0);
    pSVar24 = Packet$$GetString(incoming,(MethodInfo *)0x0);
    iVar10 = Packet$$GetShort(incoming,(MethodInfo *)0x0);
    local_d0._4_4_ = (int)iVar10;
    iVar10 = Packet$$GetShort(incoming,(MethodInfo *)0x0);
    local_d0 = CONCAT44(local_d0._4_4_,(int)iVar10);
    iVar10 = Packet$$GetShort(incoming,(MethodInfo *)0x0);
    uStack_d8._4_4_ = (int)iVar10;
    iVar10 = Packet$$GetShort(incoming,(MethodInfo *)0x0);
    uStack_d8 = CONCAT44(uStack_d8._4_4_,(int)iVar10);
    pSVar29 = Packet$$GetString(incoming,(MethodInfo *)0x0);
    if (pIVar69 != (InventoryItem_o *)0x0) {
      pSVar63 = InventoryItem$$get_item_name(pIVar69,(MethodInfo *)0x0);
      bVar5 = System.String$$op_Equality(pSVar63,StringLiteral_5977,(MethodInfo *)0x0);
      if (bVar5) {
        if ((MusicBoxControl_TypeInfo->_2).cctor_finished == 0) {
          thunk_FUN_008bc8d8();
        }
        pMVar38 = MusicBoxControl_TypeInfo->static_fields->Instance;
        pSVar115 = (System_String_array *)FUN_0083c7e4(string[]_TypeInfo,9);
        if (pSVar115 == (System_String_array *)0x0) goto LAB_00958368;
        if ((pSVar24 != (System_String_o *)0x0) &&
           (lVar61 = thunk_FUN_00818404(pSVar24,(((pSVar115->obj).klass)->_1).element_class),
           lVar61 == 0)) goto LAB_00958384;
        if ((int)pSVar115->max_length == 0) goto code_r0x0095836c;
        pSVar115->m_Items[0] = pSVar24;
        thunk_FUN_008c6b48(pSVar115->m_Items,pSVar24);
        if (StringLiteral_820 == (System_String_o *)0x0) {
          pSVar63 = (System_String_o *)0x0;
        }
        else {
          lVar61 = thunk_FUN_00818404(StringLiteral_820,(((pSVar115->obj).klass)->_1).element_class)
          ;
          pSVar63 = StringLiteral_820;
          if (lVar61 == 0) goto LAB_00958384;
        }
        if ((uint)pSVar115->max_length < 2) goto code_r0x0095836c;
        pSVar115->m_Items[1] = pSVar63;
        thunk_FUN_008c6b48(pSVar115->m_Items + 1,pSVar63);
        pSVar63 = System.Int32$$ToString((int)&local_d0 + 4,(MethodInfo *)0x0);
        if ((pSVar63 != (System_String_o *)0x0) &&
           (lVar61 = thunk_FUN_00818404(pSVar63,(((pSVar115->obj).klass)->_1).element_class),
           lVar61 == 0)) goto LAB_00958384;
        if ((uint)pSVar115->max_length < 3) {
code_r0x0095836c:
                    /* WARNING: Subroutine does not return */
          FUN_0083c8a4();
        }
        pSVar115->m_Items[2] = pSVar63;
        thunk_FUN_008c6b48(pSVar115->m_Items + 2,pSVar63);
        if (StringLiteral_820 == (System_String_o *)0x0) {
          pSVar63 = (System_String_o *)0x0;
        }
        else {
          lVar61 = thunk_FUN_00818404(StringLiteral_820,(((pSVar115->obj).klass)->_1).element_class)
          ;
          pSVar63 = StringLiteral_820;
          if (lVar61 == 0) goto LAB_00958384;
        }
        if ((uint)pSVar115->max_length < 4) goto code_r0x0095836c;
        pSVar115->m_Items[3] = pSVar63;
        thunk_FUN_008c6b48(pSVar115->m_Items + 3,pSVar63);
        pSVar63 = System.Int32$$ToString((int32_t)&local_d0,(MethodInfo *)0x0);
        if ((pSVar63 != (System_String_o *)0x0) &&
           (lVar61 = thunk_FUN_00818404(pSVar63,(((pSVar115->obj).klass)->_1).element_class),
           lVar61 == 0)) goto LAB_00958384;
        if ((uint)pSVar115->max_length < 5) goto code_r0x0095836c;
        pSVar115->m_Items[4] = pSVar63;
        thunk_FUN_008c6b48(pSVar115->m_Items + 4,pSVar63);
        if (StringLiteral_820 == (System_String_o *)0x0) {
          pSVar63 = (System_String_o *)0x0;
        }
        else {
          lVar61 = thunk_FUN_00818404(StringLiteral_820,(((pSVar115->obj).klass)->_1).element_class)
          ;
          pSVar63 = StringLiteral_820;
          if (lVar61 == 0) goto LAB_00958384;
        }
        if ((uint)pSVar115->max_length < 6) goto code_r0x0095836c;
        pSVar115->m_Items[5] = pSVar63;
        thunk_FUN_008c6b48(pSVar115->m_Items + 5,pSVar63);
        pSVar63 = System.Int32$$ToString((int)&uStack_d8 + 4,(MethodInfo *)0x0);
        if ((pSVar63 != (System_String_o *)0x0) &&
           (lVar61 = thunk_FUN_00818404(pSVar63,(((pSVar115->obj).klass)->_1).element_class),
           lVar61 == 0)) goto LAB_00958384;
        if ((uint)pSVar115->max_length < 7) goto code_r0x0095836c;
        pSVar115->m_Items[6] = pSVar63;
        thunk_FUN_008c6b48(pSVar115->m_Items + 6,pSVar63);
        if (StringLiteral_820 == (System_String_o *)0x0) {
          pSVar63 = (System_String_o *)0x0;
        }
        else {
          lVar61 = thunk_FUN_00818404(StringLiteral_820,(((pSVar115->obj).klass)->_1).element_class)
          ;
          pSVar63 = StringLiteral_820;
          if (lVar61 == 0) goto LAB_00958384;
        }
        if ((uint)pSVar115->max_length < 8) goto code_r0x0095836c;
        pSVar115->m_Items[7] = pSVar63;
        thunk_FUN_008c6b48(pSVar115->m_Items + 7,pSVar63);
        pSVar63 = System.Int32$$ToString((int32_t)&uStack_d8,(MethodInfo *)0x0);
        if ((pSVar63 != (System_String_o *)0x0) &&
           (lVar61 = thunk_FUN_00818404(pSVar63,(((pSVar115->obj).klass)->_1).element_class),
           lVar61 == 0)) goto LAB_00958384;
        if ((uint)pSVar115->max_length < 9) goto code_r0x0095836c;
        pSVar115->m_Items[8] = pSVar63;
        thunk_FUN_008c6b48(pSVar115->m_Items + 8,pSVar63);
        pSVar63 = System.String$$Concat(pSVar115,(MethodInfo *)0x0);
        if (pMVar38 == (MusicBoxControl_o *)0x0) goto LAB_00958368;
        MusicBoxControl$$remove_song(pMVar38,pSVar63,(MethodInfo *)0x0);
      }
      if ((ChunkControl_TypeInfo->_2).cctor_finished == 0) {
        thunk_FUN_008bc8d8();
      }
      pCVar120 = ChunkControl_TypeInfo->static_fields->Instance;
      if (pCVar120 != (ChunkControl_o *)0x0) {
        pSVar63 = ChunkControl$$GetChunkString
                            (pCVar120,pSVar24,local_d0._4_4_,(int32_t)local_d0,(MethodInfo *)0x0);
        pCVar120 = ChunkControl_TypeInfo->static_fields->Instance;
        if (pCVar120 != (ChunkControl_o *)0x0) {
          bVar5 = ChunkControl$$IsChunkFullyLoadedOrMidload(pCVar120,pSVar63,(MethodInfo *)0x0);
          if (bVar5) {
            pCVar104 = ConstructionControl_TypeInfo->static_fields->Instance;
          }
          else {
            if (DAT_028e6c86 == '\0') {
              FUN_0083c778(&GameServerConnector_TypeInfo);
              DAT_028e6c86 = '\x01';
            }
            pGVar90 = GameServerConnector_TypeInfo->static_fields->Instance;
            if (pGVar90 == (GameServerConnector_o *)0x0) goto LAB_00958368;
            if ((pGVar90->fields).is_host_cached == false) {
              return;
            }
            pCVar104 = ConstructionControl_TypeInfo->static_fields->Instance;
          }
          if (pCVar104 != (ConstructionControl_o *)0x0) {
            uStack_d8._4_4_ = (int)((ulong)uStack_d8 >> 0x20);
            iVar17 = uStack_d8._4_4_;
            local_d0._4_4_ = (int)((ulong)local_d0 >> 0x20);
            iVar21 = local_d0._4_4_;
            iVar22 = (int32_t)uStack_d8;
            iVar23 = (int32_t)local_d0;
            ConstructionControl$$PlayerReplaceAt
                      (pCVar104,pIVar68,pIVar69,(uint)uVar6,pSVar24,iVar21,iVar23,iVar17,iVar22,
                       false,pSVar29,(MethodInfo *)0x0);
            return;
          }
        }
      }
    }
    goto LAB_00958368;
  case 0x23:
    pSVar24 = Packet$$GetString(incoming,(MethodInfo *)0x0);
    iVar10 = Packet$$GetShort(incoming,(MethodInfo *)0x0);
    iVar11 = Packet$$GetShort(incoming,(MethodInfo *)0x0);
    iVar12 = Packet$$GetShort(incoming,(MethodInfo *)0x0);
    iVar13 = Packet$$GetShort(incoming,(MethodInfo *)0x0);
    uVar6 = Packet$$GetByte(incoming,(MethodInfo *)0x0);
    pSVar29 = Packet$$GetString(incoming,(MethodInfo *)0x0);
    pSVar115 = (System_String_array *)FUN_0083c7e4(string[]_TypeInfo,9);
    uVar32 = 0;
    ppSVar121 = pSVar115->m_Items;
    do {
      pSVar63 = Packet$$GetString(incoming,(MethodInfo *)0x0);
      if (pSVar115 == (System_String_array *)0x0) goto LAB_00958368;
      if ((pSVar63 != (System_String_o *)0x0) &&
         (lVar61 = thunk_FUN_00818404(pSVar63,(((pSVar115->obj).klass)->_1).element_class),
         lVar61 == 0)) goto LAB_00958384;
      if ((uint)pSVar115->max_length <= uVar32) goto code_r0x0095836c;
      *ppSVar121 = pSVar63;
      thunk_FUN_008c6b48(ppSVar121,pSVar63);
      uVar32 = uVar32 + 1;
      ppSVar121 = ppSVar121 + 1;
    } while (uVar32 != 9);
    if ((LandClaimControl_TypeInfo->_2).cctor_finished == 0) {
      thunk_FUN_008bc8d8();
    }
    pLVar67 = LandClaimControl_TypeInfo->static_fields->Instance;
    if (pLVar67 == (LandClaimControl_o *)0x0) goto LAB_00958368;
    LandClaimControl$$ModifyLandClaimTimer
              (pLVar67,pSVar24,(int)iVar10,(int)iVar11,(int)iVar12,(int)iVar13,(uint)uVar6,pSVar29,
               pSVar115,(MethodInfo *)0x0);
    break;
  case 0x24:
    uVar6 = Packet$$GetByte(incoming,(MethodInfo *)0x0);
    pSVar24 = Packet$$GetString(incoming,(MethodInfo *)0x0);
    if (uVar6 == 2) {
      pZVar123 = ZoneDataControl_TypeInfo->static_fields->Instance;
      if (((pZVar123 != (ZoneDataControl_o *)0x0) &&
          (pZVar77 = (pZVar123->fields).curr_zonedata_cache, pZVar77 != (ZoneData_o *)0x0)) &&
         (pSVar44 = (pZVar77->fields).outdoor_land_claim_chunk_timers,
         pSVar44 != (System_Collections_Generic_Dictionary_string__LandClaimChunkTimer__o *)0x0)) {
        System.Collections.Generic.Dictionary<>$$Remove
                  (pSVar44,pSVar24,
                   Method$System.Collections.Generic.Dictionary<string,-LandClaimChunkTimer>.Remove( )
                  );
        return;
      }
      goto LAB_00958368;
    }
    if (uVar6 != 1) {
      if (uVar6 != 0) {
        return;
      }
      iVar10 = Packet$$GetShort(incoming,(MethodInfo *)0x0);
      iVar11 = Packet$$GetShort(incoming,(MethodInfo *)0x0);
      iVar12 = Packet$$GetShort(incoming,(MethodInfo *)0x0);
      iVar13 = Packet$$GetShort(incoming,(MethodInfo *)0x0);
      pSVar29 = Packet$$GetString(incoming,(MethodInfo *)0x0);
      if ((System.DateTime_TypeInfo->_2).cctor_finished == 0) {
        thunk_FUN_008bc8d8(System.DateTime_TypeInfo);
      }
      SStack_c8._dateData = (uint64_t)System.DateTime$$get_UtcNow((MethodInfo *)0x0);
      SStack_c8._dateData =
           (uint64_t)
           System.DateTime$$AddSeconds
                     ((System_DateTime_o)&SStack_c8,(double)(int)iVar10,(MethodInfo *)0x0);
      SStack_c8._dateData =
           (uint64_t)
           System.DateTime$$AddMinutes
                     ((System_DateTime_o)&SStack_c8,(double)(int)iVar11,(MethodInfo *)0x0);
      SStack_c8._dateData =
           (uint64_t)
           System.DateTime$$AddHours
                     ((System_DateTime_o)&SStack_c8,(double)(int)iVar12,(MethodInfo *)0x0);
      SStack_c8._dateData =
           (uint64_t)
           System.DateTime$$AddDays
                     ((System_DateTime_o)&SStack_c8,(double)(int)iVar13,(MethodInfo *)0x0);
      pLVar43 = ChunkData$$CreateLandClaimChunkTimer
                          (pSVar24,pSVar29,(System_String_o *)StringLiteral_1.rgctx_data,
                           (System_String_o *)StringLiteral_1,(System_DateTime_o)SStack_c8._dateData
                           ,(MethodInfo *)0x0);
      pZVar123 = ZoneDataControl_TypeInfo->static_fields->Instance;
      if (((pZVar123 != (ZoneDataControl_o *)0x0) &&
          (pZVar77 = (pZVar123->fields).curr_zonedata_cache, pZVar77 != (ZoneData_o *)0x0)) &&
         (pSVar44 = (pZVar77->fields).outdoor_land_claim_chunk_timers,
         pSVar44 != (System_Collections_Generic_Dictionary_string__LandClaimChunkTimer__o *)0x0)) {
        uVar32 = System.Collections.Generic.Dictionary<>$$ContainsKey
                           (pSVar44,pSVar24,
                            Method$System.Collections.Generic.Dictionary<string,-LandClaimChunkTimer >.ContainsKey()
                           );
        pZVar123 = ZoneDataControl_TypeInfo->static_fields->Instance;
        if (((pZVar123 != (ZoneDataControl_o *)0x0) &&
            (pZVar77 = (pZVar123->fields).curr_zonedata_cache, pZVar77 != (ZoneData_o *)0x0)) &&
           (pSVar44 = (pZVar77->fields).outdoor_land_claim_chunk_timers,
           pSVar44 != (System_Collections_Generic_Dictionary_string__LandClaimChunkTimer__o *)0x0))
        {
          if ((uVar32 & 1) != 0) {
            System.Collections.Generic.Dictionary<>$$set_Item
                      (pSVar44,pSVar24,pLVar43,
                       Method$System.Collections.Generic.Dictionary<string,-LandClaimChunkTimer>.set _Item()
                      );
            return;
          }
          System.Collections.Generic.Dictionary<>$$Add
                    (pSVar44,pSVar24,pLVar43,
                     Method$System.Collections.Generic.Dictionary<string,-LandClaimChunkTimer>.Add()
                    );
          return;
        }
      }
      goto LAB_00958368;
    }
    uVar6 = Packet$$GetByte(incoming,(MethodInfo *)0x0);
    _Var53.rgctx_data = (Il2CppRGCTXData *)Packet$$GetString(incoming,(MethodInfo *)0x0);
    pZVar123 = ZoneDataControl_TypeInfo->static_fields->Instance;
    if (((pZVar123 == (ZoneDataControl_o *)0x0) ||
        (pZVar77 = (pZVar123->fields).curr_zonedata_cache, pZVar77 == (ZoneData_o *)0x0)) ||
       (pSVar44 = (pZVar77->fields).outdoor_land_claim_chunk_timers,
       pSVar44 == (System_Collections_Generic_Dictionary_string__LandClaimChunkTimer__o *)0x0))
    goto LAB_00958368;
    uVar32 = System.Collections.Generic.Dictionary<>$$ContainsKey
                       (pSVar44,pSVar24,
                        Method$System.Collections.Generic.Dictionary<string,-LandClaimChunkTimer>.Co ntainsKey()
                       );
    if ((uVar32 & 1) == 0) {
      return;
    }
    pZVar123 = ZoneDataControl_TypeInfo->static_fields->Instance;
    if (((pZVar123 == (ZoneDataControl_o *)0x0) ||
        (pZVar77 = (pZVar123->fields).curr_zonedata_cache, pZVar77 == (ZoneData_o *)0x0)) ||
       (pSVar44 = (pZVar77->fields).outdoor_land_claim_chunk_timers,
       pSVar44 == (System_Collections_Generic_Dictionary_string__LandClaimChunkTimer__o *)0x0))
    goto LAB_00958368;
    lVar61 = System.Collections.Generic.Dictionary<>$$get_Item
                       (pSVar44,pSVar24,
                        Method$System.Collections.Generic.Dictionary<string,-LandClaimChunkTimer>.ge t_Item()
                       );
    if (uVar6 == 2) goto joined_r0x00952178;
    if (uVar6 != 1) {
      return;
    }
    if (lVar61 == 0) goto LAB_00958368;
    p_Var73 = (_union_13 *)(lVar61 + 0x28);
    p_Var73->rgctx_data = (Il2CppRGCTXData *)_Var53;
    goto code_r0x00954f68;
  case 0x25:
    pZVar123 = ZoneDataControl_TypeInfo->static_fields->Instance;
    if ((pZVar123 != (ZoneDataControl_o *)0x0) &&
       (pZVar77 = (pZVar123->fields).curr_zonedata_cache, pZVar77 != (ZoneData_o *)0x0)) {
      ZoneData$$ClearOutdoorLandClaims(pZVar77,(MethodInfo *)0x0);
      if ((ChunkControl_TypeInfo->_2).cctor_finished == 0) {
        thunk_FUN_008bc8d8();
      }
      pCVar120 = ChunkControl_TypeInfo->static_fields->Instance;
      if (pCVar120 != (ChunkControl_o *)0x0) {
        pZVar77 = ZoneData$$UnpackFromWeb
                            (incoming,(pCVar120->fields).player_zone_cache,(MethodInfo *)0x0);
        pZVar123 = ZoneDataControl_TypeInfo->static_fields->Instance;
        if (pZVar123 != (ZoneDataControl_o *)0x0) {
          ppZVar98 = &(pZVar123->fields).curr_zonedata_cache;
          *ppZVar98 = pZVar77;
          thunk_FUN_008c6b48(ppZVar98,pZVar77);
          pZVar123 = ZoneDataControl_TypeInfo->static_fields->Instance;
          if (pZVar123 != (ZoneDataControl_o *)0x0) {
            ZoneDataControl$$UpdateZoneItemOnChangedOutside(pZVar123,(MethodInfo *)0x0);
            return;
          }
        }
      }
    }
    goto LAB_00958368;
  case 0x26:
    iVar10 = Packet$$GetShort(incoming,(MethodInfo *)0x0);
    pSVar58 = (System_Collections_Generic_List_object__o *)
              thunk_FUN_008184f0(System.Collections.Generic.List<string>_TypeInfo);
    if (pSVar58 == (System_Collections_Generic_List_object__o *)0x0) goto LAB_00958368;
    iVar20 = (int)iVar10;
    System.Collections.Generic.List<object>$$.ctor
              (pSVar58,Method$System.Collections.Generic.List<string>..ctor());
    pSVar24 = Packet$$GetString(incoming,(MethodInfo *)0x0);
    iVar19 = iVar20;
    if (0 < iVar20) {
      do {
        lVar61 = Method$System.Collections.Generic.List<string>.Add();
        pSVar114 = (pSVar58->fields)._items;
        (pSVar58->fields)._version = (pSVar58->fields)._version + 1;
        if (pSVar114 == (System_Object_array *)0x0) goto LAB_00958368;
        uVar18 = (pSVar58->fields)._size;
        if (uVar18 < (uint)pSVar114->max_length) {
          (pSVar58->fields)._size = uVar18 + 1;
          pSVar114->m_Items[(int)uVar18] = (Il2CppObject *)pSVar24;
          thunk_FUN_008c6b48(pSVar114->m_Items + (int)uVar18,pSVar24);
        }
        else {
          method_04 = *(Startup_c ***)(*(long *)(*(long *)(lVar61 + 0x20) + 0xc0) + 0x58);
          (*((MethodInfo *)method_04)->virtualMethodPointer)(pSVar58,pSVar24);
        }
        pSVar24 = Packet$$GetString(incoming,(MethodInfo *)0x0);
        iVar19 = iVar19 + -1;
      } while (iVar19 != 0);
    }
    pSVar29 = Packet$$GetString(incoming,(MethodInfo *)0x0);
    uVar6 = Packet$$GetByte(incoming,(MethodInfo *)0x0);
    pGVar40 = (GameServerReceiver_o *)(ulong)uVar6;
    if (DAT_028e6c81 == '\0') {
      pGVar40 = (GameServerReceiver_o *)FUN_0083c778(&UnityEngine.Vector3_TypeInfo);
      DAT_028e6c81 = '\x01';
    }
    if ((uVar6 & 0xfe) == 2) {
      UVar129 = GameServerReceiver$$UnpackPosition(pGVar40,incoming,(MethodInfo *)method_04);
      fVar128 = UVar129.fields.z;
      fVar127 = UVar129.fields.y;
      fVar125 = UVar129.fields.x;
    }
    else {
      pUVar110 = UnityEngine.Vector3_TypeInfo->static_fields;
      fVar125 = (pUVar110->zeroVector).fields.x;
      fVar127 = (pUVar110->zeroVector).fields.y;
      fVar128 = (pUVar110->zeroVector).fields.z;
    }
    pPVar34 = (Packet_o *)thunk_FUN_008184f0(Packet_TypeInfo);
    if (pPVar34 == (Packet_o *)0x0) goto LAB_00958368;
    Packet$$.ctor(pPVar34,(MethodInfo *)0x0);
    Packet$$PutByte(pPVar34,0x26,(MethodInfo *)0x0);
    Packet$$PutShort(pPVar34,(float)iVar20,(MethodInfo *)0x0);
    if (0 < iVar20) {
      iVar19 = 0;
      do {
        pSVar63 = (System_String_o *)
                  System.Collections.Generic.List<object>$$get_Item
                            (pSVar58,iVar19,
                             Method$System.Collections.Generic.List<string>.get_Item());
        Packet$$PutString(pPVar34,pSVar63,(MethodInfo *)0x0);
        pZVar123 = ZoneDataControl_TypeInfo->static_fields->Instance;
        pSVar63 = (System_String_o *)
                  System.Collections.Generic.List<object>$$get_Item
                            (pSVar58,iVar19,
                             Method$System.Collections.Generic.List<string>.get_Item());
        if ((pZVar123 == (ZoneDataControl_o *)0x0) ||
           (pZVar77 = ZoneDataControl$$LoadZoneDataFromDisk(pZVar123,pSVar63,(MethodInfo *)0x0),
           pZVar77 == (ZoneData_o *)0x0)) goto LAB_00958368;
        ZoneData$$PackForWeb(pZVar77,pPVar34,(MethodInfo *)0x0);
        ZoneData$$ClearOutdoorLandClaims(pZVar77,(MethodInfo *)0x0);
        iVar19 = iVar19 + 1;
      } while (iVar20 != iVar19);
    }
    Packet$$PutString(pPVar34,pSVar24,(MethodInfo *)0x0);
    Packet$$PutString(pPVar34,pSVar29,(MethodInfo *)0x0);
    method_05 = (MethodInfo *)0x0;
    Packet$$PutByte(pPVar34,uVar6,(MethodInfo *)0x0);
    pGVar106 = extraout_x0_05;
    pMVar102 = extraout_x1_21;
    if ((uVar6 & 0xfe) == 2) {
      if (DAT_028e6c85 == '\0') {
        pGVar106 = (GameServerSender_o *)FUN_0083c778(&GameServerSender_TypeInfo);
        DAT_028e6c85 = '\x01';
      }
      if (GameServerSender_TypeInfo->static_fields->Instance == (GameServerSender_o *)0x0)
      goto LAB_00958368;
      UVar130.fields.y = fVar127;
      UVar130.fields.x = fVar125;
      UVar130.fields.z = fVar128;
      GameServerSender$$PackPosition(pGVar106,pPVar34,UVar130,method_05);
      pGVar106 = extraout_x0_06;
      pMVar102 = extraout_x1_22;
    }
    goto code_r0x00957928;
  case 0x27:
    pSVar24 = Packet$$GetString(incoming,(MethodInfo *)0x0);
    _Var53.rgctx_data = (Il2CppRGCTXData *)Packet$$GetString(incoming,(MethodInfo *)0x0);
    if (DAT_028e6c84 == '\0') {
      FUN_0083c778(&GameServerInterface_TypeInfo);
      DAT_028e6c84 = '\x01';
    }
    pGVar95 = GameServerInterface_TypeInfo->static_fields->Instance;
    if ((pGVar95 == (GameServerInterface_o *)0x0) ||
       (pSVar39 = (pGVar95->fields).nearby_players,
       pSVar39 == (System_Collections_Generic_Dictionary_string__OnlinePlayer__o *)0x0))
    goto LAB_00958368;
    uVar32 = System.Collections.Generic.Dictionary<>$$ContainsKey
                       (pSVar39,pSVar24,
                        Method$System.Collections.Generic.Dictionary<string,-OnlinePlayer>.ContainsK ey()
                       );
    if ((uVar32 & 1) == 0) {
      return;
    }
    if (DAT_028e6c84 == '\0') {
      FUN_0083c778(&GameServerInterface_TypeInfo);
      DAT_028e6c84 = '\x01';
    }
    pGVar95 = GameServerInterface_TypeInfo->static_fields->Instance;
    if ((pGVar95 == (GameServerInterface_o *)0x0) ||
       (pSVar39 = (pGVar95->fields).nearby_players,
       pSVar39 == (System_Collections_Generic_Dictionary_string__OnlinePlayer__o *)0x0))
    goto LAB_00958368;
    lVar61 = System.Collections.Generic.Dictionary<>$$get_Item
                       (pSVar39,pSVar24,
                        Method$System.Collections.Generic.Dictionary<string,-OnlinePlayer>.get_Item( )
                       );
joined_r0x00952178:
    if (lVar61 == 0) goto LAB_00958368;
    p_Var73 = (_union_13 *)(lVar61 + 0x30);
    p_Var73->rgctx_data = (Il2CppRGCTXData *)_Var53;
    goto code_r0x00954f68;
  case 0x28:
    pSVar24 = Packet$$GetString(incoming,(MethodInfo *)0x0);
    if (DAT_028e6c84 == '\0') {
      FUN_0083c778(&GameServerInterface_TypeInfo);
      DAT_028e6c84 = '\x01';
    }
    pGVar95 = GameServerInterface_TypeInfo->static_fields->Instance;
    if ((pGVar95 == (GameServerInterface_o *)0x0) ||
       (pSVar39 = (pGVar95->fields).nearby_players,
       pSVar39 == (System_Collections_Generic_Dictionary_string__OnlinePlayer__o *)0x0))
    goto LAB_00958368;
    uVar32 = System.Collections.Generic.Dictionary<>$$ContainsKey
                       (pSVar39,pSVar24,
                        Method$System.Collections.Generic.Dictionary<string,-OnlinePlayer>.ContainsK ey()
                       );
    if ((uVar32 & 1) == 0) {
      return;
    }
    if (DAT_028e6c84 == '\0') {
      FUN_0083c778(&GameServerInterface_TypeInfo);
      DAT_028e6c84 = '\x01';
    }
    pGVar95 = GameServerInterface_TypeInfo->static_fields->Instance;
    if (((pGVar95 == (GameServerInterface_o *)0x0) ||
        (pSVar39 = (pGVar95->fields).nearby_players,
        pSVar39 == (System_Collections_Generic_Dictionary_string__OnlinePlayer__o *)0x0)) ||
       (lVar61 = System.Collections.Generic.Dictionary<>$$get_Item
                           (pSVar39,pSVar24,
                            Method$System.Collections.Generic.Dictionary<string,-OnlinePlayer>.get_I tem()
                           ), _Var53 = StringLiteral_1, lVar61 == 0)) goto LAB_00958368;
    p_Var73 = (_union_13 *)(lVar61 + 0x30);
    *p_Var73 = StringLiteral_1;
    goto code_r0x00954f68;
  case 0x29:
    pSVar24 = Packet$$GetString(incoming,(MethodInfo *)0x0);
    pPVar34 = (Packet_o *)thunk_FUN_008184f0(Packet_TypeInfo);
    if (pPVar34 == (Packet_o *)0x0) goto LAB_00958368;
    Packet$$.ctor(pPVar34,(MethodInfo *)0x0);
    Packet$$PutByte(pPVar34,0x2a,(MethodInfo *)0x0);
    Packet$$PutString(pPVar34,pSVar24,(MethodInfo *)0x0);
    Packet$$PutShort(pPVar34,10.0,(MethodInfo *)0x0);
    iVar19 = 10;
    do {
      pCVar104 = ConstructionControl_TypeInfo->static_fields->Instance;
      if (pCVar104 == (ConstructionControl_o *)0x0) goto LAB_00958368;
      iVar17 = ConstructionControl$$GetNewUniqueId(pCVar104,true,(MethodInfo *)0x0);
      Packet$$PutLong(pPVar34,iVar17,(MethodInfo *)0x0);
      pSVar107 = (__this->fields).unique_ids_given_away;
      if (pSVar107 == (System_Collections_Generic_Dictionary_string__List_int___o *)0x0)
      goto LAB_00958368;
      uVar32 = System.Collections.Generic.Dictionary<>$$ContainsKey
                         (pSVar107,pSVar24,
                          _Method$System.Collections.Generic.Dictionary<string,-List<int>>.ContainsK ey()
                         );
      if ((uVar32 & 1) == 0) {
        pSVar107 = (__this->fields).unique_ids_given_away;
        pSVar72 = (System_Collections_Generic_List_int__o *)
                  thunk_FUN_008184f0(System.Collections.Generic.List<int>_TypeInfo);
        if ((pSVar72 == (System_Collections_Generic_List_int__o *)0x0) ||
           (System.Collections.Generic.List<int>$$.ctor
                      (pSVar72,Method$System.Collections.Generic.List<int>..ctor()),
           pSVar107 == (System_Collections_Generic_Dictionary_string__List_int___o *)0x0))
        goto LAB_00958368;
        System.Collections.Generic.Dictionary<>$$Add
                  (pSVar107,pSVar24,pSVar72,
                   _Method$System.Collections.Generic.Dictionary<string,-List<int>>.Add());
      }
      pSVar107 = (__this->fields).unique_ids_given_away;
      if (pSVar107 == (System_Collections_Generic_Dictionary_string__List_int___o *)0x0)
      goto LAB_00958368;
      auVar132 = System.Collections.Generic.Dictionary<>$$get_Item
                           (pSVar107,pSVar24,
                            _Method$System.Collections.Generic.Dictionary<string,-List<int>>.get_Ite m()
                           );
      lVar61 = Method$System.Collections.Generic.List<int>.Add();
      lVar42 = auVar132._0_8_;
      if (lVar42 == 0) goto LAB_00958368;
      lVar96 = *(long *)(lVar42 + 0x10);
      *(int *)(lVar42 + 0x1c) = *(int *)(lVar42 + 0x1c) + 1;
      if (lVar96 == 0) goto LAB_00958368;
      uVar18 = *(uint *)(lVar42 + 0x18);
      if (uVar18 < *(uint *)(lVar96 + 0x18)) {
        *(uint *)(lVar42 + 0x18) = uVar18 + 1;
        *(int32_t *)(lVar96 + (long)(int)uVar18 * 4 + 0x20) = iVar17;
      }
      else {
        auVar132 = (**(code **)(*(long *)(*(long *)(*(long *)(lVar61 + 0x20) + 0xc0) + 0x58) + 8))
                             (lVar42,iVar17);
      }
      iVar19 = iVar19 + -1;
    } while (iVar19 != 0);
    pCVar25 = GameServerReceiver$$get_connection(auVar132._0_8_,auVar132._8_8_);
    if (pCVar25 == (Connection_o *)0x0) goto LAB_00958368;
    Connection$$Send(pCVar25,pPVar34,2,(MethodInfo *)0x0);
    pIVar57 = _StringLiteral_13258;
    if ((UnityEngine.Debug_TypeInfo->_2).cctor_finished == 0) {
      thunk_FUN_008bc8d8();
      pIVar57 = _StringLiteral_13258;
    }
    goto code_r0x00954e90;
  case 0x2a:
    iVar10 = Packet$$GetShort(incoming,(MethodInfo *)0x0);
    iVar19 = (int)iVar10;
    if (0 < iVar19) {
      do {
        pCVar104 = ConstructionControl_TypeInfo->static_fields->Instance;
        if (pCVar104 == (ConstructionControl_o *)0x0) goto LAB_00958368;
        pSVar72 = (pCVar104->fields).online_unique_ids_;
        iVar17 = Packet$$GetLong(incoming,(MethodInfo *)0x0);
        lVar61 = Method$System.Collections.Generic.List<int>.Add();
        if (pSVar72 == (System_Collections_Generic_List_int__o *)0x0) goto LAB_00958368;
        pSVar75 = (pSVar72->fields)._items;
        (pSVar72->fields)._version = (pSVar72->fields)._version + 1;
        if (pSVar75 == (System_Int32_array *)0x0) goto LAB_00958368;
        uVar18 = (pSVar72->fields)._size;
        if (uVar18 < (uint)pSVar75->max_length) {
          (pSVar72->fields)._size = uVar18 + 1;
          pSVar75->m_Items[(int)uVar18] = iVar17;
        }
        else {
          (**(code **)(*(long *)(*(long *)(*(long *)(lVar61 + 0x20) + 0xc0) + 0x58) + 8))(pSVar72);
        }
        iVar19 = iVar19 + -1;
      } while (iVar19 != 0);
    }
    pGVar106 = GameServerSender_TypeInfo->static_fields->Instance;
    if (pGVar106 == (GameServerSender_o *)0x0) goto LAB_00958368;
    (pGVar106->fields).requesting_unique_ids = false;
    break;
  case 0x2b:
    pSVar24 = Packet$$GetString(incoming,(MethodInfo *)0x0);
    iVar17 = Packet$$GetLong(incoming,(MethodInfo *)0x0);
    pSVar107 = (__this->fields).unique_ids_given_away;
    if (pSVar107 == (System_Collections_Generic_Dictionary_string__List_int___o *)0x0)
    goto LAB_00958368;
    uVar32 = System.Collections.Generic.Dictionary<>$$ContainsKey
                       (pSVar107,pSVar24,
                        _Method$System.Collections.Generic.Dictionary<string,-List<int>>.ContainsKey ()
                       );
    if ((uVar32 & 1) != 0) {
      pSVar107 = (__this->fields).unique_ids_given_away;
      if ((pSVar107 == (System_Collections_Generic_Dictionary_string__List_int___o *)0x0) ||
         (pSVar72 = (System_Collections_Generic_List_int__o *)
                    System.Collections.Generic.Dictionary<>$$get_Item
                              (pSVar107,pSVar24,
                               _Method$System.Collections.Generic.Dictionary<string,-List<int>>.get_ Item()
                              ), pSVar72 == (System_Collections_Generic_List_int__o *)0x0))
      goto LAB_00958368;
      System.Collections.Generic.List<int>$$Remove
                (pSVar72,iVar17,Method$System.Collections.Generic.List<int>.Remove());
    }
    pIVar57 = _StringLiteral_9643;
    if ((UnityEngine.Debug_TypeInfo->_2).cctor_finished == 0) {
      thunk_FUN_008bc8d8();
      pIVar57 = _StringLiteral_9643;
    }
code_r0x00954e90:
    UnityEngine.Debug$$Log(pIVar57,(MethodInfo *)0x0);
    break;
  case 0x2d:
    pSVar24 = Packet$$GetString(incoming,(MethodInfo *)0x0);
    uVar6 = Packet$$GetByte(incoming,(MethodInfo *)0x0);
    iVar10 = Packet$$GetShort(incoming,(MethodInfo *)0x0);
    iVar11 = Packet$$GetShort(incoming,(MethodInfo *)0x0);
    iVar12 = Packet$$GetShort(incoming,(MethodInfo *)0x0);
    local_b8 = CONCAT44(local_b8._4_4_,(int)iVar12);
    if (uVar6 == 0) {
      if ((MusicBoxControl_TypeInfo->_2).cctor_finished == 0) {
        thunk_FUN_008bc8d8();
      }
      pMVar38 = MusicBoxControl_TypeInfo->static_fields->Instance;
      pSVar115 = (System_String_array *)FUN_0083c7e4(string[]_TypeInfo,5);
      if (pSVar115 != (System_String_array *)0x0) {
        if ((pSVar24 != (System_String_o *)0x0) &&
           (lVar61 = thunk_FUN_00818404(pSVar24,(((pSVar115->obj).klass)->_1).element_class),
           lVar61 == 0)) {
LAB_00958384:
          uVar88 = thunk_FUN_00893bf8();
                    /* WARNING: Subroutine does not return */
          FUN_0083c864(uVar88,0);
        }
        if ((int)pSVar115->max_length == 0) goto code_r0x0095836c;
        pSVar115->m_Items[0] = pSVar24;
        thunk_FUN_008c6b48(pSVar115->m_Items,pSVar24);
        if (StringLiteral_820 == (System_String_o *)0x0) {
          pSVar24 = (System_String_o *)0x0;
        }
        else {
          lVar61 = thunk_FUN_00818404(StringLiteral_820,(((pSVar115->obj).klass)->_1).element_class)
          ;
          pSVar24 = StringLiteral_820;
          if (lVar61 == 0) goto LAB_00958384;
        }
        if (1 < (uint)pSVar115->max_length) {
          pSVar115->m_Items[1] = pSVar24;
          thunk_FUN_008c6b48(pSVar115->m_Items + 1,pSVar24);
          local_bc = (int)iVar11 + (int)iVar10;
          pSVar24 = System.Int32$$ToString((int32_t)&local_bc,(MethodInfo *)0x0);
          if ((pSVar24 != (System_String_o *)0x0) &&
             (lVar61 = thunk_FUN_00818404(pSVar24,(((pSVar115->obj).klass)->_1).element_class),
             lVar61 == 0)) goto LAB_00958384;
          if (2 < (uint)pSVar115->max_length) {
            pSVar115->m_Items[2] = pSVar24;
            thunk_FUN_008c6b48(pSVar115->m_Items + 2,pSVar24);
            if (StringLiteral_820 == (System_String_o *)0x0) {
              pSVar24 = (System_String_o *)0x0;
            }
            else {
              lVar61 = thunk_FUN_00818404(StringLiteral_820,
                                          (((pSVar115->obj).klass)->_1).element_class);
              pSVar24 = StringLiteral_820;
              if (lVar61 == 0) goto LAB_00958384;
            }
            if (3 < (uint)pSVar115->max_length) {
              pSVar115->m_Items[3] = pSVar24;
              thunk_FUN_008c6b48(pSVar115->m_Items + 3,pSVar24);
              pSVar24 = System.Int32$$ToString((int32_t)&local_b8,(MethodInfo *)0x0);
              if ((pSVar24 != (System_String_o *)0x0) &&
                 (lVar61 = thunk_FUN_00818404(pSVar24,(((pSVar115->obj).klass)->_1).element_class),
                 lVar61 == 0)) goto LAB_00958384;
              if (4 < (uint)pSVar115->max_length) {
                pSVar115->m_Items[4] = pSVar24;
                thunk_FUN_008c6b48(pSVar115->m_Items + 4,pSVar24);
                pSVar24 = System.String$$Concat(pSVar115,(MethodInfo *)0x0);
                if (pMVar38 != (MusicBoxControl_o *)0x0) {
                  MusicBoxControl$$remove_online_finger_note(pMVar38,pSVar24,(MethodInfo *)0x0);
                  return;
                }
                goto LAB_00958368;
              }
            }
          }
        }
        goto code_r0x0095836c;
      }
      goto LAB_00958368;
    }
    if (uVar6 == 1) {
      if ((MusicBoxControl_TypeInfo->_2).cctor_finished == 0) {
        thunk_FUN_008bc8d8();
      }
      pMVar38 = MusicBoxControl_TypeInfo->static_fields->Instance;
      if (pMVar38 == (MusicBoxControl_o *)0x0) goto LAB_00958368;
      MusicBoxControl$$online_finger_pressed
                (pMVar38,pSVar24,(int)iVar10,(int)iVar11,(int32_t)local_b8,(MethodInfo *)0x0);
    }
    break;
  case 0x2e:
    pSVar24 = Packet$$GetString(incoming,(MethodInfo *)0x0);
    uVar6 = Packet$$GetByte(incoming,(MethodInfo *)0x0);
    if (uVar6 == 1) {
      pSVar29 = Packet$$GetString(incoming,(MethodInfo *)0x0);
      iVar10 = Packet$$GetShort(incoming,(MethodInfo *)0x0);
      iVar11 = Packet$$GetShort(incoming,(MethodInfo *)0x0);
      iVar12 = Packet$$GetShort(incoming,(MethodInfo *)0x0);
      iVar13 = Packet$$GetShort(incoming,(MethodInfo *)0x0);
      pCVar105 = CustomTeleporterControl_TypeInfo->static_fields->Instance;
      if (pCVar105 == (CustomTeleporterControl_o *)0x0) goto LAB_00958368;
      method_06 = (MethodInfo_F19A34 **)(ulong)(uint)(int)iVar11;
      iVar17 = CustomTeleporterControl$$GetCustomTeleId
                         (pCVar105,pSVar29,(int)iVar10,(int)iVar11,(int)iVar12,(int)iVar13,
                          (MethodInfo *)0x0);
      pCVar105 = CustomTeleporterControl_TypeInfo->static_fields->Instance;
      if (pCVar105 == (CustomTeleporterControl_o *)0x0) goto LAB_00958368;
      iVar19 = CustomTeleporterControl$$FindCorrespondingTelePage(pCVar105,iVar17,(MethodInfo *)0x0)
      ;
      if (DAT_028e6c85 == '\0') {
        FUN_0083c778(&GameServerSender_TypeInfo);
        DAT_028e6c85 = '\x01';
      }
      pGVar106 = GameServerSender_TypeInfo->static_fields->Instance;
      if (pGVar106 == (GameServerSender_o *)0x0) goto LAB_00958368;
    }
    else {
      if (uVar6 != 0) {
        return;
      }
      iVar10 = Packet$$GetShort(incoming,(MethodInfo *)0x0);
      if (DAT_028e6c85 == '\0') {
        FUN_0083c778(&GameServerSender_TypeInfo);
        DAT_028e6c85 = '\x01';
      }
      pGVar106 = GameServerSender_TypeInfo->static_fields->Instance;
      if (pGVar106 == (GameServerSender_o *)0x0) goto LAB_00958368;
      iVar19 = (int)iVar10;
    }
    GameServerSender$$PackPageOfTeleporters(pGVar106,pSVar24,iVar19,(MethodInfo *)method_06);
    break;
  case 0x2f:
    pPVar52 = PopupControl_TypeInfo->static_fields->Instance;
    if (pPVar52 == (PopupControl_o *)0x0) goto LAB_00958368;
    PopupControl$$HideAll(pPVar52,(MethodInfo *)0x0);
    if ((inventory_ctr_TypeInfo->_2).cctor_finished == 0) {
      thunk_FUN_008bc8d8();
    }
    piVar112 = inventory_ctr_TypeInfo->static_fields->Instance;
    if ((piVar112 == (inventory_ctr_o *)0x0) ||
       (pUVar56 = (piVar112->fields).crafting_Tab, pUVar56 == (UnityEngine_GameObject_o *)0x0))
    goto LAB_00958368;
    UnityEngine.GameObject$$SetActive(pUVar56,true,(MethodInfo *)0x0);
    piVar112 = inventory_ctr_TypeInfo->static_fields->Instance;
    if (piVar112 == (inventory_ctr_o *)0x0) goto LAB_00958368;
    inventory_ctr$$LayOutCraftingTab(piVar112,0,(MethodInfo *)0x0);
    iVar10 = Packet$$GetShort(incoming,(MethodInfo *)0x0);
    iVar20 = (int)iVar10;
    uVar6 = Packet$$GetByte(incoming,(MethodInfo *)0x0);
    uVar8 = Packet$$GetByte(incoming,(MethodInfo *)0x0);
    iVar19 = 0;
    do {
      uVar7 = Packet$$GetByte(incoming,(MethodInfo *)0x0);
      if (uVar7 == 1) {
        auVar132 = thunk_FUN_008184f0(OnlineTeleporter_TypeInfo);
        pOVar111 = auVar132._0_8_;
        if (pOVar111 == (OnlineTeleporter_o *)0x0) goto LAB_00958368;
        OnlineTeleporter$$.ctor(pOVar111,auVar132._8_8_);
        pSVar24 = Packet$$GetString(incoming,(MethodInfo *)0x0);
        (pOVar111->fields).title = pSVar24;
        thunk_FUN_008c6b48();
        pSVar24 = Packet$$GetString(incoming,(MethodInfo *)0x0);
        (pOVar111->fields).description = pSVar24;
        thunk_FUN_008c6b48();
        pSVar24 = Packet$$GetString(incoming,(MethodInfo *)0x0);
        (pOVar111->fields).tele_str = pSVar24;
        thunk_FUN_008c6b48();
        pSVar24 = Packet$$GetString(incoming,(MethodInfo *)0x0);
        (pOVar111->fields).to_zone = pSVar24;
        thunk_FUN_008c6b48();
        iVar10 = Packet$$GetShort(incoming,(MethodInfo *)0x0);
        (pOVar111->fields).to_chunkX = (int)iVar10;
        iVar10 = Packet$$GetShort(incoming,(MethodInfo *)0x0);
        (pOVar111->fields).to_chunkZ = (int)iVar10;
        iVar10 = Packet$$GetShort(incoming,(MethodInfo *)0x0);
        (pOVar111->fields).to_innerX = (int)iVar10;
        iVar10 = Packet$$GetShort(incoming,(MethodInfo *)0x0);
        (pOVar111->fields).to_innerZ = (int)iVar10;
        pSVar24 = Packet$$GetString(incoming,(MethodInfo *)0x0);
        (pOVar111->fields).built_by = pSVar24;
        thunk_FUN_008c6b48();
        if (iVar19 == 2) {
          pCVar105 = CustomTeleporterControl_TypeInfo->static_fields->Instance;
          if (pCVar105 == (CustomTeleporterControl_o *)0x0) goto LAB_00958368;
          ppOVar36 = &(pCVar105->fields).teleporter_R;
code_r0x00950ae4:
          *ppOVar36 = pOVar111;
          thunk_FUN_008c6b48(ppOVar36,pOVar111);
        }
        else {
          if (iVar19 == 1) {
            pCVar105 = CustomTeleporterControl_TypeInfo->static_fields->Instance;
            if (pCVar105 != (CustomTeleporterControl_o *)0x0) {
              ppOVar36 = &(pCVar105->fields).teleporter_mid;
              goto code_r0x00950ae4;
            }
            goto LAB_00958368;
          }
          if (iVar19 == 0) {
            pCVar105 = CustomTeleporterControl_TypeInfo->static_fields->Instance;
            if (pCVar105 != (CustomTeleporterControl_o *)0x0) {
              ppOVar36 = &(pCVar105->fields).teleporter_L;
              goto code_r0x00950ae4;
            }
            goto LAB_00958368;
          }
        }
        if ((inventory_ctr_TypeInfo->_2).cctor_finished == 0) {
          thunk_FUN_008bc8d8();
        }
        piVar112 = inventory_ctr_TypeInfo->static_fields->Instance;
        if ((((piVar112 == (inventory_ctr_o *)0x0) ||
             (pSVar79 = (piVar112->fields).instantiated_crafting_slots,
             pSVar79 == (System_Collections_Generic_List_CraftingSlot__o *)0x0)) ||
            (pUVar37 = (UnityEngine_Component_o *)
                       System.Collections.Generic.List<object>$$get_Item
                                 ((System_Collections_Generic_List_object__o *)pSVar79,iVar19,
                                  Method$System.Collections.Generic.List<CraftingSlot>.get_Item()),
            pUVar37 == (UnityEngine_Component_o *)0x0)) ||
           (pUVar56 = UnityEngine.Component$$get_gameObject(pUVar37,(MethodInfo *)0x0),
           pUVar56 == (UnityEngine_GameObject_o *)0x0)) goto LAB_00958368;
        UnityEngine.GameObject$$SetActive(pUVar56,true,(MethodInfo *)0x0);
        pCVar105 = CustomTeleporterControl_TypeInfo->static_fields->Instance;
        if (pCVar105 == (CustomTeleporterControl_o *)0x0) goto LAB_00958368;
        CustomTeleporterControl$$DrawOnlineTeleporterSlot
                  (pCVar105,iVar19,pOVar111,(MethodInfo *)0x0);
      }
      else {
        if ((inventory_ctr_TypeInfo->_2).cctor_finished == 0) {
          thunk_FUN_008bc8d8();
        }
        piVar112 = inventory_ctr_TypeInfo->static_fields->Instance;
        if (((piVar112 == (inventory_ctr_o *)0x0) ||
            (pSVar79 = (piVar112->fields).instantiated_crafting_slots,
            pSVar79 == (System_Collections_Generic_List_CraftingSlot__o *)0x0)) ||
           ((pUVar37 = (UnityEngine_Component_o *)
                       System.Collections.Generic.List<object>$$get_Item
                                 ((System_Collections_Generic_List_object__o *)pSVar79,iVar19,
                                  Method$System.Collections.Generic.List<CraftingSlot>.get_Item()),
            pUVar37 == (UnityEngine_Component_o *)0x0 ||
            (pUVar56 = UnityEngine.Component$$get_gameObject(pUVar37,(MethodInfo *)0x0),
            pUVar56 == (UnityEngine_GameObject_o *)0x0)))) goto LAB_00958368;
        UnityEngine.GameObject$$SetActive(pUVar56,false,(MethodInfo *)0x0);
        if (iVar19 == 0) {
          pCVar105 = CustomTeleporterControl_TypeInfo->static_fields->Instance;
          if (pCVar105 == (CustomTeleporterControl_o *)0x0) goto LAB_00958368;
          ppOVar36 = &(pCVar105->fields).teleporter_L;
          *ppOVar36 = (OnlineTeleporter_o *)0x0;
code_r0x00950aac:
          thunk_FUN_008c6b48(ppOVar36,0);
        }
        else {
          if (iVar19 == 1) {
            pCVar105 = CustomTeleporterControl_TypeInfo->static_fields->Instance;
            if (pCVar105 != (CustomTeleporterControl_o *)0x0) {
              ppOVar36 = &(pCVar105->fields).teleporter_mid;
              *ppOVar36 = (OnlineTeleporter_o *)0x0;
              goto code_r0x00950aac;
            }
            goto LAB_00958368;
          }
          if (iVar19 == 2) {
            pCVar105 = CustomTeleporterControl_TypeInfo->static_fields->Instance;
            if (pCVar105 == (CustomTeleporterControl_o *)0x0) goto LAB_00958368;
            ppOVar36 = &(pCVar105->fields).teleporter_R;
            *ppOVar36 = (OnlineTeleporter_o *)0x0;
            thunk_FUN_008c6b48(ppOVar36,0);
            break;
          }
        }
      }
      iVar19 = iVar19 + 1;
    } while (iVar19 != 3);
    if (uVar6 == 1) {
      pCVar105 = CustomTeleporterControl_TypeInfo->static_fields->Instance;
      if (pCVar105 == (CustomTeleporterControl_o *)0x0) goto LAB_00958368;
      (pCVar105->fields).search_page = iVar20;
      piVar82 = inventory_ctr_TypeInfo;
    }
    else {
      if ((inventory_ctr_TypeInfo->_2).cctor_finished == 0) {
        thunk_FUN_008bc8d8();
      }
      piVar82 = inventory_ctr_TypeInfo;
      piVar112 = inventory_ctr_TypeInfo->static_fields->Instance;
      if (piVar112 == (inventory_ctr_o *)0x0) goto LAB_00958368;
      (piVar112->fields).craft_PAGE = iVar20;
    }
    if ((piVar82->_2).cctor_finished == 0) {
      thunk_FUN_008bc8d8();
      piVar82 = inventory_ctr_TypeInfo;
    }
    piVar112 = piVar82->static_fields->Instance;
    if ((piVar112 == (inventory_ctr_o *)0x0) ||
       (pUVar56 = (piVar112->fields).crafting_page_left, pUVar56 == (UnityEngine_GameObject_o *)0x0)
       ) goto LAB_00958368;
    UnityEngine.GameObject$$SetActive(pUVar56,iVar20 != 0,(MethodInfo *)0x0);
    if ((inventory_ctr_TypeInfo->_2).cctor_finished == 0) {
      thunk_FUN_008bc8d8();
    }
    piVar112 = inventory_ctr_TypeInfo->static_fields->Instance;
    if ((piVar112 == (inventory_ctr_o *)0x0) ||
       (pUVar56 = (piVar112->fields).crafting_page_right, pUVar56 == (UnityEngine_GameObject_o *)0x0
       )) goto LAB_00958368;
    UnityEngine.GameObject$$SetActive(pUVar56,uVar8 == 1,(MethodInfo *)0x0);
    pCVar105 = CustomTeleporterControl_TypeInfo->static_fields->Instance;
    if (pCVar105 == (CustomTeleporterControl_o *)0x0) goto LAB_00958368;
    if ((pCVar105->fields).in_search_screen == false) {
      return;
    }
    if ((pCVar105->fields).teleporter_L != (OnlineTeleporter_o *)0x0) {
      return;
    }
    if ((pCVar105->fields).teleporter_mid != (OnlineTeleporter_o *)0x0) {
      return;
    }
    if ((pCVar105->fields).teleporter_R != (OnlineTeleporter_o *)0x0) {
      return;
    }
    if (iVar20 != 0) {
      return;
    }
    pPVar52 = PopupControl_TypeInfo->static_fields->Instance;
    if (pPVar52 == (PopupControl_o *)0x0) goto LAB_00958368;
    puVar100 = (undefined8 *)&StringLiteral_1082;
    goto code_r0x00953554;
  case 0x30:
    pSVar24 = Packet$$GetString(incoming,(MethodInfo *)0x0);
    iVar10 = Packet$$GetShort(incoming,(MethodInfo *)0x0);
    iVar11 = Packet$$GetShort(incoming,(MethodInfo *)0x0);
    iVar12 = Packet$$GetShort(incoming,(MethodInfo *)0x0);
    iVar13 = Packet$$GetShort(incoming,(MethodInfo *)0x0);
    uVar18 = Packet$$GetLong(incoming,(MethodInfo *)0x0);
    pSVar50 = (System_Byte_array *)FUN_0083c7e4(byte[]_TypeInfo,(ulong)uVar18);
    if (0 < (int)uVar18) {
      uVar32 = 0;
      do {
        uVar6 = Packet$$GetByte(incoming,(MethodInfo *)0x0);
        if (pSVar50 == (System_Byte_array *)0x0) goto LAB_00958368;
        if ((uint)pSVar50->max_length <= uVar32) goto code_r0x0095836c;
        pSVar50->m_Items[uVar32] = uVar6;
        uVar32 = uVar32 + 1;
      } while (uVar18 != uVar32);
    }
    pCVar105 = CustomTeleporterControl_TypeInfo->static_fields->Instance;
    if (pCVar105 == (CustomTeleporterControl_o *)0x0) goto LAB_00958368;
    iVar17 = CustomTeleporterControl$$GetCustomTeleId
                       (pCVar105,pSVar24,(int)iVar10,(int)iVar11,(int)iVar12,(int)iVar13,
                        (MethodInfo *)0x0);
    local_b8 = CONCAT44(iVar17,(int32_t)local_b8);
    if (iVar17 != -1) {
      pSVar24 = Startup_TypeInfo->static_fields->persistentDataPath;
      if ((System.IO.Path_TypeInfo->_2).cctor_finished == 0) {
        thunk_FUN_008bc8d8();
      }
      if ((char_TypeInfo->_2).cctor_finished == 0) {
        thunk_FUN_008bc8d8();
      }
      pSVar29 = System.Char$$ToString
                          ((short)System.IO.Path_TypeInfo->static_fields + 10,(MethodInfo *)0x0);
      pPVar64 = PlayerData_TypeInfo->static_fields->Instance;
      if (pPVar64 == (PlayerData_o *)0x0) goto LAB_00958368;
      pSVar63 = PlayerData$$GetCurrentSlotFolder(pPVar64,(MethodInfo *)0x0);
      pSVar24 = System.String$$Concat(pSVar24,pSVar29,pSVar63,(MethodInfo *)0x0);
      pSVar29 = System.Int32$$ToString((int)&local_b8 + 4,(MethodInfo *)0x0);
      pSVar29 = System.String$$Concat(StringLiteral_13664,pSVar29,(MethodInfo *)0x0);
      pSVar24 = System.IO.Path$$Combine(pSVar24,pSVar29,(MethodInfo *)0x0);
      System.IO.File$$WriteAllBytes(pSVar24,pSVar50,(MethodInfo *)0x0);
    }
    break;
  case 0x31:
    pSVar24 = Packet$$GetString(incoming,(MethodInfo *)0x0);
    pSVar29 = Packet$$GetString(incoming,(MethodInfo *)0x0);
    iVar10 = Packet$$GetShort(incoming,(MethodInfo *)0x0);
    iVar11 = Packet$$GetShort(incoming,(MethodInfo *)0x0);
    iVar12 = Packet$$GetShort(incoming,(MethodInfo *)0x0);
    iVar13 = Packet$$GetShort(incoming,(MethodInfo *)0x0);
    pCVar105 = CustomTeleporterControl_TypeInfo->static_fields->Instance;
    if (pCVar105 == (CustomTeleporterControl_o *)0x0) goto LAB_00958368;
    iVar17 = CustomTeleporterControl$$GetCustomTeleId
                       (pCVar105,pSVar29,(int)iVar10,(int)iVar11,(int)iVar12,(int)iVar13,
                        (MethodInfo *)0x0);
    uStack_b0 = CONCAT44(uStack_b0._4_4_,iVar17);
    if (iVar17 == -1) {
      return;
    }
    pSVar63 = Startup_TypeInfo->static_fields->persistentDataPath;
    if ((System.IO.Path_TypeInfo->_2).cctor_finished == 0) {
      thunk_FUN_008bc8d8();
    }
    if ((char_TypeInfo->_2).cctor_finished == 0) {
      thunk_FUN_008bc8d8();
    }
    pSVar65 = System.Char$$ToString
                        ((short)System.IO.Path_TypeInfo->static_fields + 10,(MethodInfo *)0x0);
    pPVar64 = PlayerData_TypeInfo->static_fields->Instance;
    if (pPVar64 == (PlayerData_o *)0x0) goto LAB_00958368;
    str2 = PlayerData$$GetCurrentSlotFolder(pPVar64,(MethodInfo *)0x0);
    pSVar63 = System.String$$Concat(pSVar63,pSVar65,str2,(MethodInfo *)0x0);
    pSVar65 = System.Int32$$ToString((int32_t)&uStack_b0,(MethodInfo *)0x0);
    pSVar65 = System.String$$Concat(StringLiteral_13664,pSVar65,(MethodInfo *)0x0);
    pSVar65 = System.IO.Path$$Combine(pSVar63,pSVar65,(MethodInfo *)0x0);
    bVar5 = System.IO.File$$Exists(pSVar65,(MethodInfo *)0x0);
    if (!bVar5) {
      return;
    }
    pPVar34 = (Packet_o *)thunk_FUN_008184f0(Packet_TypeInfo);
    if (pPVar34 == (Packet_o *)0x0) goto LAB_00958368;
    Packet$$.ctor(pPVar34,(MethodInfo *)0x0);
    Packet$$PutByte(pPVar34,0x32,(MethodInfo *)0x0);
    Packet$$PutString(pPVar34,pSVar24,(MethodInfo *)0x0);
    Packet$$PutString(pPVar34,pSVar29,(MethodInfo *)0x0);
    Packet$$PutShort(pPVar34,(float)(int)iVar10,(MethodInfo *)0x0);
    Packet$$PutShort(pPVar34,(float)(int)iVar11,(MethodInfo *)0x0);
    Packet$$PutShort(pPVar34,(float)(int)iVar12,(MethodInfo *)0x0);
    Packet$$PutShort(pPVar34,(float)(int)iVar13,(MethodInfo *)0x0);
    pSVar24 = System.Int32$$ToString((int32_t)&uStack_b0,(MethodInfo *)0x0);
    pSVar24 = System.String$$Concat(StringLiteral_13664,pSVar24,(MethodInfo *)0x0);
    if ((System.IO.Path_TypeInfo->_2).cctor_finished == 0) {
      thunk_FUN_008bc8d8(System.IO.Path_TypeInfo);
    }
    pSVar24 = System.IO.Path$$Combine(pSVar63,pSVar24,(MethodInfo *)0x0);
    pSVar50 = System.IO.File$$ReadAllBytes(pSVar24,(MethodInfo *)0x0);
    if (pSVar50 == (System_Byte_array *)0x0) goto LAB_00958368;
    Packet$$PutLong(pPVar34,(int32_t)pSVar50->max_length,(MethodInfo *)0x0);
    pGVar40 = extraout_x0_02;
    pMVar102 = extraout_x1_02;
    if (0 < (int)pSVar50->max_length) {
      uVar32 = 0;
      uVar97 = pSVar50->max_length & 0xffffffff;
      do {
        if (uVar97 <= uVar32) goto code_r0x0095836c;
        Packet$$PutByte(pPVar34,pSVar50->m_Items[uVar32],(MethodInfo *)0x0);
        uVar18 = (uint)pSVar50->max_length;
        uVar97 = (ulong)uVar18;
        uVar32 = uVar32 + 1;
        pGVar40 = extraout_x0_03;
        pMVar102 = extraout_x1_03;
      } while ((long)uVar32 < (long)(int)uVar18);
    }
    pCVar25 = GameServerReceiver$$get_connection(pGVar40,pMVar102);
    if (pCVar25 == (Connection_o *)0x0) goto LAB_00958368;
    iVar17 = 3;
    goto code_r0x00957938;
  case 0x32:
    pSVar24 = Packet$$GetString(incoming,(MethodInfo *)0x0);
    uVar18 = Packet$$GetLong(incoming,(MethodInfo *)0x0);
    pSVar50 = (System_Byte_array *)FUN_0083c7e4(byte[]_TypeInfo,(ulong)uVar18);
    fVar127 = (float)in_d5;
    fVar125 = (float)in_d4;
    if (0 < (int)uVar18) {
      uVar32 = 0;
      do {
        uVar6 = Packet$$GetByte(incoming,(MethodInfo *)0x0);
        fVar127 = (float)in_d5;
        fVar125 = (float)in_d4;
        if (pSVar50 == (System_Byte_array *)0x0) goto LAB_00958368;
        if ((uint)pSVar50->max_length <= uVar32) goto code_r0x0095836c;
        pSVar50->m_Items[uVar32] = uVar6;
        uVar32 = uVar32 + 1;
      } while (uVar18 != uVar32);
    }
    __this_08 = (UnityEngine_Texture2D_o *)thunk_FUN_008184f0(UnityEngine.Texture2D_TypeInfo);
    if (__this_08 != (UnityEngine_Texture2D_o *)0x0) {
      UnityEngine.Texture2D$$.ctor(__this_08,0x96,0x96,(MethodInfo *)0x0);
      UnityEngine.ImageConversion$$LoadImage(__this_08,pSVar50,(MethodInfo *)0x0);
      iVar19 = (*(__this_08->klass->vtable)._4_get_width.methodPtr)
                         (__this_08,(__this_08->klass->vtable)._4_get_width.method);
      iVar20 = (*(__this_08->klass->vtable)._6_get_height.methodPtr)
                         (__this_08,(__this_08->klass->vtable)._6_get_height.method);
      auStack_188._0_8_ = (System_Collections_Generic_List_T__o *)0x0;
      auStack_188._8_8_ = (Il2CppType **)0x0;
      auVar132._4_4_ = (float)iVar20;
      auVar132._0_4_ = (float)iVar19;
      auVar132._8_8_ = 0;
      UnityEngine.Rect$$.ctor
                ((UnityEngine_Rect_o)(auVar132 << 0x40),fVar125,fVar127,in_s6,in_s7,
                 (MethodInfo *)auStack_188);
      rect.fields.m_Width = (float)auStack_188._8_4_;
      rect.fields.m_Height = (float)auStack_188._12_4_;
      rect.fields.m_XMin = (float)auStack_188._0_4_;
      rect.fields.m_YMin = (float)auStack_188._4_4_;
      pUVar48 = UnityEngine.Sprite$$Create
                          (__this_08,rect,(UnityEngine_Vector2_o)0x3f0000003f000000,
                           (MethodInfo *)0x0);
      pSVar99 = (__this->fields).cached_teleporter_textures;
      if (pSVar99 != (System_Collections_Generic_Dictionary_string__Sprite__o *)0x0) {
        uVar32 = System.Collections.Generic.Dictionary<>$$ContainsKey
                           (pSVar99,pSVar24,
                            Method$System.Collections.Generic.Dictionary<string,-Sprite>.ContainsKey ()
                           );
        pSVar99 = (__this->fields).cached_teleporter_textures;
        if (pSVar99 != (System_Collections_Generic_Dictionary_string__Sprite__o *)0x0) {
          if ((uVar32 & 1) == 0) {
            System.Collections.Generic.Dictionary<>$$Add
                      (pSVar99,pSVar24,pUVar48,
                       Method$System.Collections.Generic.Dictionary<string,-Sprite>.Add());
          }
          else {
            System.Collections.Generic.Dictionary<>$$set_Item
                      (pSVar99,pSVar24,pUVar48,
                       Method$System.Collections.Generic.Dictionary<string,-Sprite>.set_Item());
          }
          pWVar51 = WindowControl_TypeInfo->static_fields->Instance;
          if (pWVar51 != (WindowControl_o *)0x0) {
            if ((pWVar51->fields).curr_miniwindow != 3) {
              return;
            }
            pCVar105 = CustomTeleporterControl_TypeInfo->static_fields->Instance;
            if ((pCVar105 != (CustomTeleporterControl_o *)0x0) &&
               (pOVar111 = (pCVar105->fields).teleporter_L, pOVar111 != (OnlineTeleporter_o *)0x0))
            {
              bVar5 = System.String$$op_Equality
                                ((pOVar111->fields).tele_str,pSVar24,(MethodInfo *)0x0);
              if (bVar5) {
                if ((inventory_ctr_TypeInfo->_2).cctor_finished == 0) {
                  thunk_FUN_008bc8d8();
                }
                piVar112 = inventory_ctr_TypeInfo->static_fields->Instance;
                if (((piVar112 == (inventory_ctr_o *)0x0) ||
                    (pSVar79 = (piVar112->fields).instantiated_crafting_slots,
                    pSVar79 == (System_Collections_Generic_List_CraftingSlot__o *)0x0)) ||
                   (pIVar57 = System.Collections.Generic.List<object>$$get_Item
                                        ((System_Collections_Generic_List_object__o *)pSVar79,0,
                                         Method$System.Collections.Generic.List<CraftingSlot>.get_It em()
                                        ), pIVar57 == (Il2CppObject *)0x0)) goto LAB_00958368;
                pSVar99 = (__this->fields).cached_teleporter_textures;
              }
              else {
                pCVar105 = CustomTeleporterControl_TypeInfo->static_fields->Instance;
                if ((pCVar105 == (CustomTeleporterControl_o *)0x0) ||
                   (pOVar111 = (pCVar105->fields).teleporter_mid,
                   pOVar111 == (OnlineTeleporter_o *)0x0)) goto LAB_00958368;
                bVar5 = System.String$$op_Equality
                                  ((pOVar111->fields).tele_str,pSVar24,(MethodInfo *)0x0);
                if (bVar5) {
                  if ((inventory_ctr_TypeInfo->_2).cctor_finished == 0) {
                    thunk_FUN_008bc8d8();
                  }
                  piVar112 = inventory_ctr_TypeInfo->static_fields->Instance;
                  if ((piVar112 == (inventory_ctr_o *)0x0) ||
                     (pSVar79 = (piVar112->fields).instantiated_crafting_slots,
                     pSVar79 == (System_Collections_Generic_List_CraftingSlot__o *)0x0))
                  goto LAB_00958368;
                  iVar17 = 1;
                }
                else {
                  pCVar105 = CustomTeleporterControl_TypeInfo->static_fields->Instance;
                  if ((pCVar105 == (CustomTeleporterControl_o *)0x0) ||
                     (pOVar111 = (pCVar105->fields).teleporter_R,
                     pOVar111 == (OnlineTeleporter_o *)0x0)) goto LAB_00958368;
                  bVar5 = System.String$$op_Equality
                                    ((pOVar111->fields).tele_str,pSVar24,(MethodInfo *)0x0);
                  if (!bVar5) {
                    return;
                  }
                  if ((inventory_ctr_TypeInfo->_2).cctor_finished == 0) {
                    thunk_FUN_008bc8d8();
                  }
                  piVar112 = inventory_ctr_TypeInfo->static_fields->Instance;
                  if ((piVar112 == (inventory_ctr_o *)0x0) ||
                     (pSVar79 = (piVar112->fields).instantiated_crafting_slots,
                     pSVar79 == (System_Collections_Generic_List_CraftingSlot__o *)0x0))
                  goto LAB_00958368;
                  iVar17 = 2;
                }
                pIVar57 = System.Collections.Generic.List<object>$$get_Item
                                    ((System_Collections_Generic_List_object__o *)pSVar79,iVar17,
                                     Method$System.Collections.Generic.List<CraftingSlot>.get_Item()
                                    );
                if (pIVar57 == (Il2CppObject *)0x0) goto LAB_00958368;
                pSVar99 = (__this->fields).cached_teleporter_textures;
              }
              if (pSVar99 != (System_Collections_Generic_Dictionary_string__Sprite__o *)0x0) {
                pIVar54 = pIVar57[2].klass;
                pUVar48 = (UnityEngine_Sprite_o *)
                          System.Collections.Generic.Dictionary<>$$get_Item
                                    (pSVar99,pSVar24,
                                     Method$System.Collections.Generic.Dictionary<string,-Sprite>.ge t_Item()
                                    );
                if (pIVar54 != (Il2CppClass *)0x0) {
                  UnityEngine.UI.Image$$set_sprite
                            ((UnityEngine_UI_Image_o *)pIVar54,pUVar48,(MethodInfo *)0x0);
                  return;
                }
              }
            }
          }
        }
      }
    }
    goto LAB_00958368;
  case 0x33:
    pSVar24 = Packet$$GetString(incoming,(MethodInfo *)0x0);
    pSVar29 = Packet$$GetString(incoming,(MethodInfo *)0x0);
    pSVar63 = Packet$$GetString(incoming,(MethodInfo *)0x0);
    iVar10 = Packet$$GetShort(incoming,(MethodInfo *)0x0);
    iVar11 = Packet$$GetShort(incoming,(MethodInfo *)0x0);
    iVar12 = Packet$$GetShort(incoming,(MethodInfo *)0x0);
    iVar13 = Packet$$GetShort(incoming,(MethodInfo *)0x0);
    pCVar105 = CustomTeleporterControl_TypeInfo->static_fields->Instance;
    if (pCVar105 != (CustomTeleporterControl_o *)0x0) {
      iVar17 = CustomTeleporterControl$$GetCustomTeleId
                         (pCVar105,pSVar63,(int)iVar10,(int)iVar11,(int)iVar12,(int)iVar13,
                          (MethodInfo *)0x0);
      uStack_b0 = CONCAT44(iVar17,(undefined4)uStack_b0);
      if (iVar17 == -1) {
        return;
      }
      pPVar64 = PlayerData_TypeInfo->static_fields->Instance;
      pSVar63 = System.Int32$$ToString((int)&uStack_b0 + 4,(MethodInfo *)0x0);
      pSVar63 = System.String$$Concat
                          (StringLiteral_13671,pSVar63,StringLiteral_9947,(MethodInfo *)0x0);
      if (pPVar64 != (PlayerData_o *)0x0) {
        PlayerData$$SetSlotString
                  (pPVar64,pSVar63,pSVar24,4,StringLiteral_10841,-1,(MethodInfo *)0x0);
        pPVar64 = PlayerData_TypeInfo->static_fields->Instance;
        pSVar24 = System.Int32$$ToString((int)&uStack_b0 + 4,(MethodInfo *)0x0);
        pSVar24 = System.String$$Concat
                            (StringLiteral_13671,pSVar24,StringLiteral_9917,(MethodInfo *)0x0);
        if (pPVar64 != (PlayerData_o *)0x0) {
          PlayerData$$SetSlotString
                    (pPVar64,pSVar24,pSVar29,4,StringLiteral_10841,-1,(MethodInfo *)0x0);
          return;
        }
      }
    }
    goto LAB_00958368;
  case 0x35:
    pSVar24 = Packet$$GetString(incoming,(MethodInfo *)0x0);
    uVar6 = Packet$$GetByte(incoming,(MethodInfo *)0x0);
    if (uVar6 < 2) {
      pMVar122 = MinigameMenu_TypeInfo->static_fields->Instance;
      if ((UnityEngine.Object_TypeInfo->_2).cctor_finished == 0) {
        thunk_FUN_008bc8d8();
      }
      bVar5 = UnityEngine.Object$$op_Inequality
                        ((UnityEngine_Object_o *)pMVar122,(UnityEngine_Object_o *)0x0,
                         (MethodInfo *)0x0);
      if (!bVar5) {
        return;
      }
      pMVar122 = MinigameMenu_TypeInfo->static_fields->Instance;
      if (pMVar122 == (MinigameMenu_o *)0x0) goto LAB_00958368;
      iVar19 = (pMVar122->fields).curr_menu;
      if (iVar19 == 6) {
        if (DAT_028e6c85 == '\0') {
          FUN_0083c778(&GameServerSender_TypeInfo);
          DAT_028e6c85 = '\x01';
        }
        pGVar106 = GameServerSender_TypeInfo->static_fields->Instance;
        if (pGVar106 == (GameServerSender_o *)0x0) goto LAB_00958368;
        uVar8 = 2;
      }
      else if (iVar19 == 5) {
        if (DAT_028e6c85 == '\0') {
          FUN_0083c778(&GameServerSender_TypeInfo);
          DAT_028e6c85 = '\x01';
        }
        pGVar106 = GameServerSender_TypeInfo->static_fields->Instance;
        if (pGVar106 == (GameServerSender_o *)0x0) goto LAB_00958368;
        uVar8 = 1;
      }
      else {
        if (DAT_028e6c85 == '\0') {
          FUN_0083c778(&GameServerSender_TypeInfo);
          DAT_028e6c85 = '\x01';
        }
        pGVar106 = GameServerSender_TypeInfo->static_fields->Instance;
        if (pGVar106 == (GameServerSender_o *)0x0) goto LAB_00958368;
        if (iVar19 == 3) {
          uVar8 = 3;
        }
        else {
          uVar8 = 0;
        }
      }
    }
    else {
      if (uVar6 != 2) {
        return;
      }
      pTVar117 = TradingTableControl_TypeInfo->static_fields->Instance;
      if ((UnityEngine.Object_TypeInfo->_2).cctor_finished == 0) {
        thunk_FUN_008bc8d8();
      }
      bVar5 = UnityEngine.Object$$op_Inequality
                        ((UnityEngine_Object_o *)pTVar117,(UnityEngine_Object_o *)0x0,
                         (MethodInfo *)0x0);
      if (!bVar5) {
        return;
      }
      pTVar117 = TradingTableControl_TypeInfo->static_fields->Instance;
      if (pTVar117 == (TradingTableControl_o *)0x0) goto LAB_00958368;
      bVar1 = (pTVar117->fields).other_player_has_joined;
      if (DAT_028e6c85 == '\0') {
        FUN_0083c778(&GameServerSender_TypeInfo);
        DAT_028e6c85 = '\x01';
      }
      pGVar106 = GameServerSender_TypeInfo->static_fields->Instance;
      if (pGVar106 == (GameServerSender_o *)0x0) goto LAB_00958368;
      if (bVar1 == false) {
        uVar8 = 3;
      }
      else {
        uVar8 = 2;
      }
      uVar6 = 2;
    }
    GameServerSender$$SendMinigameResponse(pGVar106,uVar8,pSVar24,uVar6,(MethodInfo *)method_07);
    break;
  case 0x36:
    pIVar57 = (Il2CppObject *)thunk_FUN_008184f0(GameServerReceiver.<>c__DisplayClass18_0_TypeInfo);
    if (pIVar57 == (Il2CppObject *)0x0) goto LAB_00958368;
    System.Object$$.ctor(pIVar57,(MethodInfo *)0x0);
    uVar6 = Packet$$GetByte(incoming,(MethodInfo *)0x0);
    pIVar54 = (Il2CppClass *)Packet$$GetString(incoming,(MethodInfo *)0x0);
    pIVar119 = pIVar57 + 1;
    pIVar119->klass = pIVar54;
    thunk_FUN_008c6b48(pIVar119,pIVar54);
    uVar8 = Packet$$GetByte(incoming,(MethodInfo *)0x0);
    *(uint8_t *)&pIVar57[1].monitor = uVar8;
    if (uVar8 < 2) {
      switch(uVar6) {
      case 0:
        pPVar109 = PopupControl_TypeInfo->static_fields;
        puVar100 = (undefined8 *)&StringLiteral_1421;
        break;
      case 1:
        pPVar109 = PopupControl_TypeInfo->static_fields;
        puVar100 = (undefined8 *)&StringLiteral_1416;
        break;
      case 2:
        pPVar109 = PopupControl_TypeInfo->static_fields;
        puVar100 = (undefined8 *)&StringLiteral_1417;
        break;
      case 3:
        pPVar52 = PopupControl_TypeInfo->static_fields->Instance;
        pSVar87 = (System_Action_o *)thunk_FUN_008184f0(System.Action_TypeInfo);
        if ((pSVar87 == (System_Action_o *)0x0) ||
           (System.Action$$.ctor(), pPVar52 == (PopupControl_o *)0x0)) goto LAB_00958368;
        puVar100 = (undefined8 *)&StringLiteral_6647;
code_r0x009580bc:
        ppSVar118 = &(pPVar52->fields).on_yes_pressed;
        *ppSVar118 = pSVar87;
        thunk_FUN_008c6b48(ppSVar118,pSVar87);
        pPVar52 = PopupControl_TypeInfo->static_fields->Instance;
        pSVar24 = System.String$$Concat
                            ((System_String_o *)*puVar100,(System_String_o *)pIVar119->klass,
                             StringLiteral_1776,(MethodInfo *)0x0);
        if (pPVar52 == (PopupControl_o *)0x0) goto LAB_00958368;
        PopupControl$$ShowYesNo
                  (pPVar52,pSVar24,StringLiteral_9499,StringLiteral_6100,0xd,(MethodInfo *)0x0);
      default:
        goto LAB_0094fa28;
      }
    }
    else {
      if (uVar8 != 2) {
        return;
      }
      if (uVar6 == 3) {
        pPVar52 = PopupControl_TypeInfo->static_fields->Instance;
        pSVar87 = (System_Action_o *)thunk_FUN_008184f0(System.Action_TypeInfo);
        if ((pSVar87 == (System_Action_o *)0x0) ||
           (System.Action$$.ctor(), pPVar52 == (PopupControl_o *)0x0)) goto LAB_00958368;
        puVar100 = (undefined8 *)&StringLiteral_8671;
        goto code_r0x009580bc;
      }
      if (uVar6 != 2) {
        return;
      }
      pPVar109 = PopupControl_TypeInfo->static_fields;
      puVar100 = (undefined8 *)&StringLiteral_1418;
    }
    pIVar54 = pIVar119->klass;
    pPVar52 = pPVar109->Instance;
    pSVar29 = (System_String_o *)*puVar100;
    pSVar24 = StringLiteral_1565;
    goto code_r0x00957fb4;
  case 0x37:
    pIVar54 = (Il2CppClass *)Packet$$GetString(incoming,(MethodInfo *)0x0);
    uVar6 = Packet$$GetByte(incoming,(MethodInfo *)0x0);
    uVar8 = Packet$$GetByte(incoming,(MethodInfo *)0x0);
    switch(uVar6) {
    case 0:
      pPVar52 = PopupControl_TypeInfo->static_fields->Instance;
      pSVar24 = _StringLiteral_3346;
      pSVar29 = _StringLiteral_1424;
      goto code_r0x00957fb4;
    case 1:
      pPVar52 = PopupControl_TypeInfo->static_fields->Instance;
      pSVar24 = _StringLiteral_3327;
      pSVar29 = _StringLiteral_1409;
code_r0x00957fb4:
      pSVar24 = System.String$$Concat(pSVar24,(System_String_o *)pIVar54,pSVar29,(MethodInfo *)0x0);
      if (pPVar52 == (PopupControl_o *)0x0) goto LAB_00958368;
      goto code_r0x00957fc8;
    case 2:
      if (uVar8 == 0) {
        pSVar75 = (System_Int32_array *)FUN_0083c7e4(int[]_TypeInfo,0xe);
        uVar32 = 0;
        do {
          uVar6 = Packet$$GetByte(incoming,(MethodInfo *)0x0);
          if (pSVar75 == (System_Int32_array *)0x0) goto LAB_00958368;
          if ((uint)pSVar75->max_length <= uVar32) goto code_r0x0095836c;
          pSVar75->m_Items[uVar32] = (uint)uVar6;
          uVar32 = uVar32 + 1;
        } while (uVar32 != 0xe);
        pPVar52 = PopupControl_TypeInfo->static_fields->Instance;
        if (pPVar52 != (PopupControl_o *)0x0) {
          PopupControl$$HideAll(pPVar52,(MethodInfo *)0x0);
          if ((GameController_TypeInfo->_2).cctor_finished == 0) {
            thunk_FUN_008bc8d8();
          }
          pGVar93 = GameController_TypeInfo->static_fields->Instance;
          if (pGVar93 != (GameController_o *)0x0) {
            GameController$$OpenPoolTable
                      (pGVar93,(System_String_o *)pIVar54,pSVar75,(MethodInfo *)0x0);
            return;
          }
        }
        goto LAB_00958368;
      }
      break;
    case 3:
      pMVar122 = MinigameMenu_TypeInfo->static_fields->Instance;
      if ((UnityEngine.Object_TypeInfo->_2).cctor_finished == 0) {
        thunk_FUN_008bc8d8();
      }
      bVar5 = UnityEngine.Object$$op_Inequality
                        ((UnityEngine_Object_o *)pMVar122,(UnityEngine_Object_o *)0x0,
                         (MethodInfo *)0x0);
      if (bVar5) {
        pMVar122 = MinigameMenu_TypeInfo->static_fields->Instance;
        if (pMVar122 == (MinigameMenu_o *)0x0) goto LAB_00958368;
        if ((pMVar122->fields).curr_menu == 3) {
          if (uVar8 != 0) {
            return;
          }
          pSVar75 = (System_Int32_array *)FUN_0083c7e4(int[]_TypeInfo,0xe);
          uVar32 = 0;
          do {
            uVar6 = Packet$$GetByte(incoming,(MethodInfo *)0x0);
            if (pSVar75 == (System_Int32_array *)0x0) goto LAB_00958368;
            if ((uint)pSVar75->max_length <= uVar32) goto code_r0x0095836c;
            pSVar75->m_Items[uVar32] = (uint)uVar6;
            uVar32 = uVar32 + 1;
          } while (uVar32 != 0xe);
          if ((PoolGameControl_TypeInfo->_2).cctor_finished == 0) {
            thunk_FUN_008bc8d8();
          }
          pPVar116 = PoolGameControl_TypeInfo->static_fields->Instance;
          if ((UnityEngine.Object_TypeInfo->_2).cctor_finished == 0) {
            thunk_FUN_008bc8d8(UnityEngine.Object_TypeInfo);
          }
          bVar5 = UnityEngine.Object$$op_Inequality
                            ((UnityEngine_Object_o *)pPVar116,(UnityEngine_Object_o *)0x0,
                             (MethodInfo *)0x0);
          if (bVar5) {
            if ((PoolGameControl_TypeInfo->_2).cctor_finished == 0) {
              thunk_FUN_008bc8d8();
            }
            pPVar116 = PoolGameControl_TypeInfo->static_fields->Instance;
            if (pPVar116 == (PoolGameControl_o *)0x0) goto LAB_00958368;
            ppSVar121 = &(pPVar116->fields).curr_opponent;
            *ppSVar121 = (System_String_o *)pIVar54;
            thunk_FUN_008c6b48(ppSVar121,pIVar54);
            pPVar116 = PoolGameControl_TypeInfo->static_fields->Instance;
            if (pPVar116 == (PoolGameControl_o *)0x0) goto LAB_00958368;
            PoolGameControl$$ArrangeBalls(pPVar116,pSVar75,(MethodInfo *)0x0);
            pPVar116 = PoolGameControl_TypeInfo->static_fields->Instance;
            if (pPVar116 == (PoolGameControl_o *)0x0) goto LAB_00958368;
            PoolGameControl$$StartMpGame(pPVar116,true,(MethodInfo *)0x0);
          }
          if (DAT_028e6c85 == '\0') {
            FUN_0083c778(&GameServerSender_TypeInfo);
            DAT_028e6c85 = '\x01';
          }
          pGVar106 = GameServerSender_TypeInfo->static_fields->Instance;
          if (pGVar106 != (GameServerSender_o *)0x0) {
            GameServerSender$$SendBeginMinigame
                      (pGVar106,(System_String_o *)pIVar54,2,0,pSVar75,(MethodInfo *)method_08);
            return;
          }
          goto LAB_00958368;
        }
        if (DAT_028e6c85 == '\0') {
          FUN_0083c778(&GameServerSender_TypeInfo);
          DAT_028e6c85 = '\x01';
        }
        pGVar106 = GameServerSender_TypeInfo->static_fields->Instance;
        if (pGVar106 == (GameServerSender_o *)0x0) goto LAB_00958368;
        uVar6 = 1;
      }
      else {
        if (DAT_028e6c85 == '\0') {
          FUN_0083c778(&GameServerSender_TypeInfo);
          DAT_028e6c85 = '\x01';
        }
        pGVar106 = GameServerSender_TypeInfo->static_fields->Instance;
        if (pGVar106 == (GameServerSender_o *)0x0) goto LAB_00958368;
        uVar6 = 0;
      }
      GameServerSender$$SendBeginMinigame
                (pGVar106,(System_String_o *)pIVar54,uVar6,uVar8,(System_Int32_array *)0x0,
                 (MethodInfo *)method_08);
    }
    break;
  case 0x38:
    if ((PoolGameControl_TypeInfo->_2).cctor_finished == 0) {
      thunk_FUN_008bc8d8();
    }
    pPVar116 = PoolGameControl_TypeInfo->static_fields->Instance;
    if ((UnityEngine.Object_TypeInfo->_2).cctor_finished == 0) {
      thunk_FUN_008bc8d8(UnityEngine.Object_TypeInfo);
    }
    bVar5 = UnityEngine.Object$$op_Inequality
                      ((UnityEngine_Object_o *)pPVar116,(UnityEngine_Object_o *)0x0,
                       (MethodInfo *)0x0);
    if (!bVar5) {
      return;
    }
    if ((PoolGameControl_TypeInfo->_2).cctor_finished == 0) {
      thunk_FUN_008bc8d8();
    }
    pPVar116 = PoolGameControl_TypeInfo->static_fields->Instance;
    if (pPVar116 == (PoolGameControl_o *)0x0) goto LAB_00958368;
    (pPVar116->fields).show_ad_on_close = false;
    pWVar51 = WindowControl_TypeInfo->static_fields->Instance;
    if (pWVar51 == (WindowControl_o *)0x0) goto LAB_00958368;
    WindowControl$$PressClose(pWVar51,(MethodInfo *)0x0);
    pPVar52 = PopupControl_TypeInfo->static_fields->Instance;
    if (pPVar52 == (PopupControl_o *)0x0) goto LAB_00958368;
    puVar100 = (undefined8 *)&StringLiteral_6442;
code_r0x00953554:
    pSVar24 = (System_String_o *)*puVar100;
code_r0x00957fc8:
    PopupControl$$ShowMessage(pPVar52,pSVar24,1,(MethodInfo *)0x0);
    break;
  case 0x39:
    if ((PoolGameControl_TypeInfo->_2).cctor_finished == 0) {
      thunk_FUN_008bc8d8();
    }
    pPVar116 = PoolGameControl_TypeInfo->static_fields->Instance;
    if ((UnityEngine.Object_TypeInfo->_2).cctor_finished == 0) {
      thunk_FUN_008bc8d8(UnityEngine.Object_TypeInfo);
    }
    bVar5 = UnityEngine.Object$$op_Inequality
                      ((UnityEngine_Object_o *)pPVar116,(UnityEngine_Object_o *)0x0,
                       (MethodInfo *)0x0);
    if (bVar5) {
      iVar17 = Packet$$GetLong(incoming,(MethodInfo *)0x0);
      if ((PoolGameControl_TypeInfo->_2).cctor_finished == 0) {
        thunk_FUN_008bc8d8(PoolGameControl_TypeInfo);
      }
      pPVar116 = PoolGameControl_TypeInfo->static_fields->Instance;
      if (pPVar116 == (PoolGameControl_o *)0x0) goto LAB_00958368;
      PoolGameControl$$TryUpdateCuePosition(pPVar116,(float)iVar17 / 100.0,(MethodInfo *)0x0);
    }
    break;
  case 0x3a:
    if ((PoolGameControl_TypeInfo->_2).cctor_finished == 0) {
      thunk_FUN_008bc8d8();
    }
    pPVar116 = PoolGameControl_TypeInfo->static_fields->Instance;
    if ((UnityEngine.Object_TypeInfo->_2).cctor_finished == 0) {
      thunk_FUN_008bc8d8(UnityEngine.Object_TypeInfo);
    }
    bVar5 = UnityEngine.Object$$op_Inequality
                      ((UnityEngine_Object_o *)pPVar116,(UnityEngine_Object_o *)0x0,
                       (MethodInfo *)0x0);
    if (bVar5) {
      iVar17 = Packet$$GetLong(incoming,(MethodInfo *)0x0);
      iVar10 = Packet$$GetShort(incoming,(MethodInfo *)0x0);
      uVar18 = Packet$$GetLong(incoming,(MethodInfo *)0x0);
      pSVar50 = (System_Byte_array *)FUN_0083c7e4(byte[]_TypeInfo,(ulong)uVar18);
      if (0 < (int)uVar18) {
        uVar32 = 0;
        do {
          uVar6 = Packet$$GetByte(incoming,(MethodInfo *)0x0);
          if (pSVar50 == (System_Byte_array *)0x0) goto LAB_00958368;
          if ((uint)pSVar50->max_length <= uVar32) goto code_r0x0095836c;
          pSVar50->m_Items[uVar32] = uVar6;
          uVar32 = uVar32 + 1;
        } while (uVar18 != uVar32);
      }
      __this_09 = (PoolGameRecording_o *)thunk_FUN_008184f0(PoolGameRecording_TypeInfo);
      if (__this_09 != (PoolGameRecording_o *)0x0) {
        PoolGameRecording$$.ctor(__this_09,(MethodInfo *)0x0);
        PoolGameRecording$$unpack_from_web(__this_09,pSVar50,(MethodInfo *)0x0);
        if ((PoolGameControl_TypeInfo->_2).cctor_finished == 0) {
          thunk_FUN_008bc8d8();
        }
        pPVar116 = PoolGameControl_TypeInfo->static_fields->Instance;
        if (pPVar116 != (PoolGameControl_o *)0x0) {
          (pPVar116->fields).MP_next_deg = (float)iVar17 / 100.0;
          (pPVar116->fields).MP_next_power = (float)(int)iVar10 / 100.0;
          (pPVar116->fields).MP_next_recording = __this_09;
          thunk_FUN_008c6b48(&(pPVar116->fields).MP_next_recording,__this_09);
          pPVar116 = PoolGameControl_TypeInfo->static_fields->Instance;
          if (pPVar116 != (PoolGameControl_o *)0x0) {
            if ((pPVar116->fields).GamePhase != 0x12) {
              return;
            }
            if ((PoolGameControl_TypeInfo->_2).cctor_finished == 0) {
              thunk_FUN_008bc8d8(PoolGameControl_TypeInfo);
              pPVar116 = PoolGameControl_TypeInfo->static_fields->Instance;
              if (pPVar116 == (PoolGameControl_o *)0x0) goto LAB_00958368;
            }
            PoolGameControl$$ShowMpRecording(pPVar116,(MethodInfo *)0x0);
            return;
          }
        }
      }
      goto LAB_00958368;
    }
    break;
  case 0x3b:
    if ((PoolGameControl_TypeInfo->_2).cctor_finished == 0) {
      thunk_FUN_008bc8d8();
    }
    pPVar116 = PoolGameControl_TypeInfo->static_fields->Instance;
    if ((UnityEngine.Object_TypeInfo->_2).cctor_finished == 0) {
      thunk_FUN_008bc8d8(UnityEngine.Object_TypeInfo);
    }
    bVar5 = UnityEngine.Object$$op_Inequality
                      ((UnityEngine_Object_o *)pPVar116,(UnityEngine_Object_o *)0x0,
                       (MethodInfo *)0x0);
    if (bVar5) {
      if ((PoolGameControl_TypeInfo->_2).cctor_finished == 0) {
        thunk_FUN_008bc8d8();
      }
      pPVar116 = PoolGameControl_TypeInfo->static_fields->Instance;
      if (pPVar116 == (PoolGameControl_o *)0x0) goto LAB_00958368;
      PoolGameControl$$OnOtherPlayerReady(pPVar116,(MethodInfo *)0x0);
    }
    break;
  case 0x3c:
    if ((PoolGameControl_TypeInfo->_2).cctor_finished == 0) {
      thunk_FUN_008bc8d8();
    }
    pPVar116 = PoolGameControl_TypeInfo->static_fields->Instance;
    if ((UnityEngine.Object_TypeInfo->_2).cctor_finished == 0) {
      thunk_FUN_008bc8d8(UnityEngine.Object_TypeInfo);
    }
    bVar5 = UnityEngine.Object$$op_Inequality
                      ((UnityEngine_Object_o *)pPVar116,(UnityEngine_Object_o *)0x0,
                       (MethodInfo *)0x0);
    if (bVar5) {
      iVar17 = Packet$$GetLong(incoming,(MethodInfo *)0x0);
      iVar21 = Packet$$GetLong(incoming,(MethodInfo *)0x0);
      if ((PoolGameControl_TypeInfo->_2).cctor_finished == 0) {
        thunk_FUN_008bc8d8(PoolGameControl_TypeInfo);
      }
      pPVar116 = PoolGameControl_TypeInfo->static_fields->Instance;
      if (pPVar116 == (PoolGameControl_o *)0x0) goto LAB_00958368;
      V.fields.x = (float)iVar17;
      V.fields.y = (float)iVar21;
      PoolGameControl$$PlaceWhiteBallAt(pPVar116,V,(MethodInfo *)0x0);
    }
    break;
  case 0x3d:
    pSVar75 = (System_Int32_array *)FUN_0083c7e4(int[]_TypeInfo,0xe);
    uVar32 = 0;
    do {
      uVar6 = Packet$$GetByte(incoming,(MethodInfo *)0x0);
      if (pSVar75 == (System_Int32_array *)0x0) goto LAB_00958368;
      if ((uint)pSVar75->max_length <= uVar32) goto code_r0x0095836c;
      pSVar75->m_Items[uVar32] = (uint)uVar6;
      uVar32 = uVar32 + 1;
    } while (uVar32 != 0xe);
    if ((PoolGameControl_TypeInfo->_2).cctor_finished == 0) {
      thunk_FUN_008bc8d8();
    }
    pPVar116 = PoolGameControl_TypeInfo->static_fields->Instance;
    if ((UnityEngine.Object_TypeInfo->_2).cctor_finished == 0) {
      thunk_FUN_008bc8d8(UnityEngine.Object_TypeInfo);
    }
    bVar5 = UnityEngine.Object$$op_Inequality
                      ((UnityEngine_Object_o *)pPVar116,(UnityEngine_Object_o *)0x0,
                       (MethodInfo *)0x0);
    if (bVar5) {
      if ((PoolGameControl_TypeInfo->_2).cctor_finished == 0) {
        thunk_FUN_008bc8d8();
      }
      pPVar116 = PoolGameControl_TypeInfo->static_fields->Instance;
      if (pPVar116 != (PoolGameControl_o *)0x0) {
        PoolGameControl$$ArrangeBalls(pPVar116,pSVar75,(MethodInfo *)0x0);
        pPVar116 = PoolGameControl_TypeInfo->static_fields->Instance;
        if (pPVar116 != (PoolGameControl_o *)0x0) {
          PoolGameControl$$RestartMpGame(pPVar116,(MethodInfo *)0x0);
          return;
        }
      }
      goto LAB_00958368;
    }
    break;
  case 0x3e:
    pSVar24 = Packet$$GetString(incoming,(MethodInfo *)0x0);
    pSVar29 = Packet$$GetString(incoming,(MethodInfo *)0x0);
    pGVar95 = GameServerInterface_TypeInfo->static_fields->Instance;
    if (pGVar95 == (GameServerInterface_o *)0x0) goto LAB_00958368;
    pUVar74 = (UnityEngine_Object_o *)
              GameServerInterface$$GetPlayerByUsername(pGVar95,pSVar24,(MethodInfo *)method_04);
    if ((UnityEngine.Object_TypeInfo->_2).cctor_finished == 0) {
      thunk_FUN_008bc8d8(UnityEngine.Object_TypeInfo);
    }
    bVar5 = UnityEngine.Object$$op_Inequality(pUVar74,(UnityEngine_Object_o *)0x0,(MethodInfo *)0x0)
    ;
    if (bVar5) {
      bVar5 = System.String$$op_Inequality
                        (pSVar29,(System_String_o *)StringLiteral_1.rgctx_data,(MethodInfo *)0x0);
      if ((pUVar74 == (UnityEngine_Object_o *)0x0) ||
         (pSVar26 = (SharedCreature_o *)
                    UnityEngine.GameObject$$GetComponent<object>
                              ((UnityEngine_GameObject_o *)pUVar74,
                               Method$UnityEngine.GameObject.GetComponent<SharedCreature>()),
         pSVar26 == (SharedCreature_o *)0x0)) goto LAB_00958368;
      if (bVar5) {
        SharedCreature$$TrySitInChairObj(pSVar26,pSVar29,(MethodInfo *)0x0);
      }
      else {
        SharedCreature$$EndSittingInChair(pSVar26,(MethodInfo *)0x0);
      }
    }
    break;
  case 0x3f:
    uVar6 = Packet$$GetByte(incoming,(MethodInfo *)0x0);
    uVar18 = (uint)uVar6;
    if (uVar6 != 0) {
      do {
        pSVar24 = Packet$$GetString(incoming,(MethodInfo *)0x0);
        uVar6 = Packet$$GetByte(incoming,(MethodInfo *)0x0);
        if (uVar6 == 1) {
          if (DAT_028e6c85 == '\0') {
            FUN_0083c778(&GameServerSender_TypeInfo);
            DAT_028e6c85 = '\x01';
          }
          pGVar106 = GameServerSender_TypeInfo->static_fields->Instance;
          if ((pGVar106 == (GameServerSender_o *)0x0) ||
             (pSVar35 = (pGVar106->fields).mobs_I_am_trying_to_claim_awaiting_response,
             pSVar35 == (System_Collections_Generic_Dictionary_string__CreatureStruct__o *)0x0))
          goto LAB_00958368;
          uVar32 = System.Collections.Generic.Dictionary<>$$ContainsKey
                             (pSVar35,pSVar24,
                              Method$System.Collections.Generic.Dictionary<string,-CreatureStruct>.C ontainsKey()
                             );
          if ((uVar32 & 1) != 0) {
            pMVar103 = MobControl_TypeInfo->static_fields->Instance;
            if ((pMVar103 == (MobControl_o *)0x0) ||
               (pSVar55 = (pMVar103->fields).active_combatants,
               pSVar55 == (System_Collections_Generic_Dictionary_string__GameObject__o *)0x0))
            goto LAB_00958368;
            uVar32 = System.Collections.Generic.Dictionary<>$$ContainsKey
                               (pSVar55,pSVar24,
                                Method$System.Collections.Generic.Dictionary<string,-GameObject>.Con tainsKey()
                               );
            if ((uVar32 & 1) == 0) {
              if (DAT_028e6c85 == '\0') {
                FUN_0083c778(&GameServerSender_TypeInfo);
                DAT_028e6c85 = '\x01';
              }
              pGVar106 = GameServerSender_TypeInfo->static_fields->Instance;
              if ((((pGVar106 == (GameServerSender_o *)0x0) ||
                   (pSVar35 = (pGVar106->fields).mobs_I_am_trying_to_claim_awaiting_response,
                   pSVar35 == (System_Collections_Generic_Dictionary_string__CreatureStruct__o *)0x0
                   )) || (pCVar60 = (CreatureStruct_o *)
                                    System.Collections.Generic.Dictionary<>$$get_Item
                                              (pSVar35,pSVar24,
                                               _Method$System.Collections.Generic.Dictionary<string, -CreatureStruct>.get_Item()
                                              ), pCVar60 == (CreatureStruct_o *)0x0)) ||
                 (pMVar103 = MobControl_TypeInfo->static_fields->Instance,
                 pMVar103 == (MobControl_o *)0x0)) goto LAB_00958368;
              UVar129.fields.x =
                   (float)((pCVar60->fields).original_element_chunkX * 10) +
                   (float)(pCVar60->fields).original_element_innerX +
                   (float)(pCVar60->fields).spawn_offset_x + 0.5;
              UVar129.fields.z =
                   (float)((pCVar60->fields).original_element_chunkZ * 10) +
                   (float)(pCVar60->fields).original_element_innerZ +
                   (float)(pCVar60->fields).spawn_offset_z + 0.5;
              UVar129.fields.y = 1.5;
              MobControl$$SpawnLocalMob(pMVar103,pCVar60,UVar129,(MethodInfo *)0x0);
            }
          }
        }
        if (DAT_028e6c85 == '\0') {
          FUN_0083c778(&GameServerSender_TypeInfo);
          DAT_028e6c85 = '\x01';
        }
        pGVar106 = GameServerSender_TypeInfo->static_fields->Instance;
        if ((pGVar106 == (GameServerSender_o *)0x0) ||
           (pSVar35 = (pGVar106->fields).mobs_I_am_trying_to_claim_awaiting_response,
           pSVar35 == (System_Collections_Generic_Dictionary_string__CreatureStruct__o *)0x0))
        goto LAB_00958368;
        System.Collections.Generic.Dictionary<>$$Remove
                  (pSVar35,pSVar24,
                   _Method$System.Collections.Generic.Dictionary<string,-CreatureStruct>.Remove());
        uVar18 = uVar18 - 1;
      } while (uVar18 != 0);
    }
    break;
  case 0x40:
    pSVar24 = Packet$$GetString(incoming,(MethodInfo *)0x0);
    pMVar103 = MobControl_TypeInfo->static_fields->Instance;
    if ((pMVar103 == (MobControl_o *)0x0) ||
       (pSVar55 = (pMVar103->fields).active_combatants,
       pSVar55 == (System_Collections_Generic_Dictionary_string__GameObject__o *)0x0))
    goto LAB_00958368;
    uVar32 = System.Collections.Generic.Dictionary<>$$ContainsKey
                       (pSVar55,pSVar24,
                        Method$System.Collections.Generic.Dictionary<string,-GameObject>.ContainsKey ()
                       );
    if ((uVar32 & 1) == 0) {
      return;
    }
    pMVar103 = MobControl_TypeInfo->static_fields->Instance;
    if ((pMVar103 == (MobControl_o *)0x0) ||
       (pSVar55 = (pMVar103->fields).active_combatants,
       pSVar55 == (System_Collections_Generic_Dictionary_string__GameObject__o *)0x0))
    goto LAB_00958368;
    pUVar74 = (UnityEngine_Object_o *)
              System.Collections.Generic.Dictionary<>$$get_Item
                        (pSVar55,pSVar24,
                         Method$System.Collections.Generic.Dictionary<string,-GameObject>.get_Item()
                        );
    if ((ChunkControl_TypeInfo->_2).cctor_finished == 0) {
      thunk_FUN_008bc8d8(ChunkControl_TypeInfo);
    }
    if (pUVar74 == (UnityEngine_Object_o *)0x0) goto LAB_00958368;
    pCVar120 = ChunkControl_TypeInfo->static_fields->Instance;
    pUVar59 = UnityEngine.GameObject$$get_transform
                        ((UnityEngine_GameObject_o *)pUVar74,(MethodInfo *)0x0);
    if ((pUVar59 == (UnityEngine_Transform_o *)0x0) ||
       (UVar129 = UnityEngine.Transform$$get_position(pUVar59,(MethodInfo *)0x0),
       pCVar120 == (ChunkControl_o *)0x0)) goto LAB_00958368;
    pSVar29 = ChunkControl$$GetChunkString(pCVar120,UVar129,(MethodInfo *)0x0);
    pIVar57 = UnityEngine.GameObject$$GetComponent<object>
                        ((UnityEngine_GameObject_o *)pUVar74,
                         Method$UnityEngine.GameObject.GetComponent<SharedCreature>());
    if (pIVar57 == (Il2CppObject *)0x0) goto LAB_00958368;
    if (*(int *)&pIVar57[0xb].monitor != 9) {
      pIVar57 = UnityEngine.GameObject$$GetComponent<object>
                          ((UnityEngine_GameObject_o *)pUVar74,
                           Method$UnityEngine.GameObject.GetComponent<SharedCreature>());
      if (pIVar57 == (Il2CppObject *)0x0) goto LAB_00958368;
      if (*(int *)&pIVar57[0xb].monitor != 6) {
        if ((ChunkControl_TypeInfo->_2).cctor_finished == 0) {
          thunk_FUN_008bc8d8();
        }
        pCVar120 = ChunkControl_TypeInfo->static_fields->Instance;
        if (pCVar120 == (ChunkControl_o *)0x0) goto LAB_00958368;
        bVar5 = ChunkControl$$IsChunkFullyLoadedOrMidload(pCVar120,pSVar29,(MethodInfo *)0x0);
        if (bVar5) {
          pPVar34 = (Packet_o *)thunk_FUN_008184f0(Packet_TypeInfo);
          if (pPVar34 == (Packet_o *)0x0) goto LAB_00958368;
          Packet$$.ctor(pPVar34,(MethodInfo *)0x0);
          Packet$$PutByte(pPVar34,0x44,(MethodInfo *)0x0);
          Packet$$PutString(pPVar34,pSVar24,(MethodInfo *)0x0);
          pCVar25 = GameServerReceiver$$get_connection(__this_16,method_03);
          if (pCVar25 == (Connection_o *)0x0) goto LAB_00958368;
          iVar17 = 2;
          goto code_r0x00957938;
        }
      }
    }
    if ((UnityEngine.Object_TypeInfo->_2).cctor_finished == 0) {
      thunk_FUN_008bc8d8();
    }
    goto code_r0x00956fe0;
  case 0x41:
    pSVar58 = (System_Collections_Generic_List_object__o *)
              thunk_FUN_008184f0(System.Collections.Generic.List<string>_TypeInfo);
    if (pSVar58 == (System_Collections_Generic_List_object__o *)0x0) goto LAB_00958368;
    System.Collections.Generic.List<object>$$.ctor
              (pSVar58,Method$System.Collections.Generic.List<string>..ctor());
    pSVar24 = Packet$$GetString(incoming,(MethodInfo *)0x0);
    pGVar95 = GameServerInterface_TypeInfo->static_fields->Instance;
    if ((pGVar95 == (GameServerInterface_o *)0x0) ||
       (pSVar39 = (pGVar95->fields).nearby_players,
       pSVar39 == (System_Collections_Generic_Dictionary_string__OnlinePlayer__o *)0x0))
    goto LAB_00958368;
    pMVar102 = Method$System.Collections.Generic.Dictionary<string,-OnlinePlayer>.ContainsKey();
    uVar32 = System.Collections.Generic.Dictionary<>$$ContainsKey(pSVar39,pSVar24);
    if ((uVar32 & 1) == 0) {
      bVar5 = false;
    }
    else {
      pGVar95 = GameServerInterface_TypeInfo->static_fields->Instance;
      if (((pGVar95 == (GameServerInterface_o *)0x0) ||
          (pSVar39 = (pGVar95->fields).nearby_players,
          pSVar39 == (System_Collections_Generic_Dictionary_string__OnlinePlayer__o *)0x0)) ||
         (pMVar102 = Method$System.Collections.Generic.Dictionary<string,-OnlinePlayer>.get_Item(),
         lVar61 = System.Collections.Generic.Dictionary<>$$get_Item(pSVar39,pSVar24), lVar61 == 0))
      goto LAB_00958368;
      iVar19 = *(int *)(lVar61 + 0x28);
      bVar5 = iVar19 == 0;
      if (bVar5) {
        iVar19 = 5;
        *(undefined4 *)(lVar61 + 0x28) = 5;
      }
      *(int *)(lVar61 + 0x28) = iVar19 + -1;
    }
    uVar6 = Packet$$GetByte(incoming,(MethodInfo *)0x0);
    uVar18 = (uint)uVar6;
    if (uVar6 != 0) {
      do {
        pGVar40 = (GameServerReceiver_o *)Packet$$GetString(incoming,(MethodInfo *)0x0);
        UVar129 = GameServerReceiver$$UnpackPosition(pGVar40,incoming,pMVar102);
        UVar130 = GameServerReceiver$$UnpackPosition(__this_14,incoming,pMVar102);
        UVar131 = GameServerReceiver$$UnpackRotation(__this_15,incoming,pMVar102);
        pMVar103 = MobControl_TypeInfo->static_fields->Instance;
        if ((pMVar103 == (MobControl_o *)0x0) ||
           (pSVar55 = (pMVar103->fields).active_combatants,
           pSVar55 == (System_Collections_Generic_Dictionary_string__GameObject__o *)0x0))
        goto LAB_00958368;
        uVar32 = System.Collections.Generic.Dictionary<>$$ContainsKey
                           (pSVar55,pGVar40,
                            Method$System.Collections.Generic.Dictionary<string,-GameObject>.Contain sKey()
                           );
        if ((uVar32 & 1) == 0) {
          if (DAT_028e6c85 == '\0') {
            FUN_0083c778(&GameServerSender_TypeInfo);
            DAT_028e6c85 = '\x01';
          }
          pGVar106 = GameServerSender_TypeInfo->static_fields->Instance;
          if ((pGVar106 == (GameServerSender_o *)0x0) ||
             (pSVar27 = (pGVar106->fields).other_players_mobs_that_I_inquired_about,
             pSVar27 == (System_Collections_Generic_List_string__o *)0x0)) goto LAB_00958368;
          pMVar102 = Method$System.Collections.Generic.List<string>.Contains();
          bVar9 = System.Collections.Generic.List<object>$$Contains
                            ((System_Collections_Generic_List_object__o *)pSVar27,
                             (Il2CppObject *)pGVar40,
                             (MethodInfo_F1A0AC *)
                             Method$System.Collections.Generic.List<string>.Contains());
          lVar61 = Method$System.Collections.Generic.List<string>.Add();
          if (!bVar9) {
            pSVar114 = (pSVar58->fields)._items;
            (pSVar58->fields)._version = (pSVar58->fields)._version + 1;
            if (pSVar114 == (System_Object_array *)0x0) goto LAB_00958368;
            uVar2 = (pSVar58->fields)._size;
            if (uVar2 < (uint)pSVar114->max_length) {
              (pSVar58->fields)._size = uVar2 + 1;
              pSVar114->m_Items[(int)uVar2] = (Il2CppObject *)pGVar40;
              thunk_FUN_008c6b48(pSVar114->m_Items + (int)uVar2,pGVar40);
            }
            else {
              pMVar102 = *(MethodInfo **)(*(long *)(*(long *)(lVar61 + 0x20) + 0xc0) + 0x58);
              (*pMVar102->virtualMethodPointer)(pSVar58,pGVar40);
            }
            if (DAT_028e6c85 == '\0') {
              FUN_0083c778(&GameServerSender_TypeInfo);
              DAT_028e6c85 = '\x01';
            }
            lVar61 = Method$System.Collections.Generic.List<string>.Add();
            pGVar106 = GameServerSender_TypeInfo->static_fields->Instance;
            if ((pGVar106 == (GameServerSender_o *)0x0) ||
               (pSVar27 = (pGVar106->fields).other_players_mobs_that_I_inquired_about,
               pSVar27 == (System_Collections_Generic_List_string__o *)0x0)) goto LAB_00958368;
            pSVar115 = (pSVar27->fields)._items;
            (pSVar27->fields)._version = (pSVar27->fields)._version + 1;
            if (pSVar115 == (System_String_array *)0x0) goto LAB_00958368;
            uVar2 = (pSVar27->fields)._size;
            if (uVar2 < (uint)pSVar115->max_length) {
              (pSVar27->fields)._size = uVar2 + 1;
              pSVar115->m_Items[(int)uVar2] = (System_String_o *)pGVar40;
              thunk_FUN_008c6b48(pSVar115->m_Items + (int)uVar2,pGVar40);
            }
            else {
              pMVar102 = *(MethodInfo **)(*(long *)(*(long *)(lVar61 + 0x20) + 0xc0) + 0x58);
              (*pMVar102->virtualMethodPointer)(pSVar27,pGVar40);
            }
          }
        }
        else {
          pMVar103 = MobControl_TypeInfo->static_fields->Instance;
          if ((pMVar103 == (MobControl_o *)0x0) ||
             (pSVar55 = (pMVar103->fields).active_combatants,
             pSVar55 == (System_Collections_Generic_Dictionary_string__GameObject__o *)0x0))
          goto LAB_00958368;
          pUVar74 = (UnityEngine_Object_o *)
                    System.Collections.Generic.Dictionary<>$$get_Item
                              (pSVar55,pGVar40,
                               Method$System.Collections.Generic.Dictionary<string,-GameObject>.get_ Item()
                              );
          if ((UnityEngine.Object_TypeInfo->_2).cctor_finished == 0) {
            thunk_FUN_008bc8d8(UnityEngine.Object_TypeInfo);
          }
          pMVar102 = (MethodInfo *)0x0;
          bVar9 = UnityEngine.Object$$op_Inequality
                            (pUVar74,(UnityEngine_Object_o *)0x0,(MethodInfo *)0x0);
          if (bVar9) {
            if (bVar5) {
              if (GameServerInterface_TypeInfo->static_fields->Instance ==
                  (GameServerInterface_o *)0x0) goto LAB_00958368;
              GameServerInterface$$CreateMovementSmoother
                        ((GameServerInterface_o *)0x1,(UnityEngine_GameObject_o *)pUVar74,UVar129,
                         UVar130,pMVar102);
            }
            if ((pUVar74 == (UnityEngine_Object_o *)0x0) ||
               (pSVar26 = (SharedCreature_o *)
                          UnityEngine.GameObject$$GetComponent<object>
                                    ((UnityEngine_GameObject_o *)pUVar74,
                                     Method$UnityEngine.GameObject.GetComponent<SharedCreature>()),
               pSVar26 == (SharedCreature_o *)0x0)) goto LAB_00958368;
            SharedCreature$$SetMoveTo(pSVar26,UVar130,(MethodInfo *)0x0);
            pSVar26 = (SharedCreature_o *)
                      UnityEngine.GameObject$$GetComponent<object>
                                ((UnityEngine_GameObject_o *)pUVar74,
                                 Method$UnityEngine.GameObject.GetComponent<SharedCreature>());
            if (pSVar26 == (SharedCreature_o *)0x0) goto LAB_00958368;
            SharedCreature$$SnapSpotterRotation(pSVar26,UVar131,(MethodInfo *)0x0);
          }
        }
        uVar18 = uVar18 - 1;
      } while (uVar18 != 0);
    }
    if ((pSVar58->fields)._size == 0) {
      return;
    }
    pPVar34 = (Packet_o *)thunk_FUN_008184f0(Packet_TypeInfo);
    if (pPVar34 == (Packet_o *)0x0) goto LAB_00958368;
    Packet$$.ctor(pPVar34,(MethodInfo *)0x0);
    Packet$$PutByte(pPVar34,0x42,(MethodInfo *)0x0);
    Packet$$PutString(pPVar34,pSVar24,(MethodInfo *)0x0);
    Packet$$PutByte(pPVar34,(uint8_t)(pSVar58->fields)._size,(MethodInfo *)0x0);
    pGVar106 = extraout_x0_07;
    pMVar102 = extraout_x1_26;
    if (0 < (pSVar58->fields)._size) {
      iVar19 = 0;
      do {
        pSVar24 = (System_String_o *)
                  System.Collections.Generic.List<object>$$get_Item
                            (pSVar58,iVar19,
                             Method$System.Collections.Generic.List<string>.get_Item());
        Packet$$PutString(pPVar34,pSVar24,(MethodInfo *)0x0);
        iVar19 = iVar19 + 1;
        pGVar106 = extraout_x0_08;
        pMVar102 = extraout_x1_27;
      } while (iVar19 < (pSVar58->fields)._size);
    }
code_r0x00957928:
    pCVar25 = GameServerReceiver$$get_connection((GameServerReceiver_o *)pGVar106,pMVar102);
joined_r0x0095792c:
    if (pCVar25 == (Connection_o *)0x0) goto LAB_00958368;
    iVar17 = 2;
    goto code_r0x00957938;
  case 0x42:
    pSVar24 = Packet$$GetString(incoming,(MethodInfo *)0x0);
    pPVar34 = (Packet_o *)thunk_FUN_008184f0(Packet_TypeInfo);
    if (pPVar34 == (Packet_o *)0x0) goto LAB_00958368;
    Packet$$.ctor(pPVar34,(MethodInfo *)0x0);
    Packet$$PutByte(pPVar34,0x43,(MethodInfo *)0x0);
    Packet$$PutString(pPVar34,pSVar24,(MethodInfo *)0x0);
    uVar6 = Packet$$GetByte(incoming,(MethodInfo *)0x0);
    Packet$$PutByte(pPVar34,uVar6,(MethodInfo *)0x0);
    pGVar40 = extraout_x0;
    pMVar102 = extraout_x1;
    for (uVar18 = (uint)uVar6; uVar18 != 0; uVar18 = uVar18 - 1) {
      pSVar24 = Packet$$GetString(incoming,(MethodInfo *)0x0);
      Packet$$PutString(pPVar34,pSVar24,(MethodInfo *)0x0);
      pMVar103 = MobControl_TypeInfo->static_fields->Instance;
      if ((pMVar103 == (MobControl_o *)0x0) ||
         (pSVar55 = (pMVar103->fields).active_combatants,
         pSVar55 == (System_Collections_Generic_Dictionary_string__GameObject__o *)0x0))
      goto LAB_00958368;
      uVar32 = System.Collections.Generic.Dictionary<>$$ContainsKey
                         (pSVar55,pSVar24,
                          Method$System.Collections.Generic.Dictionary<string,-GameObject>.ContainsK ey()
                         );
      if ((uVar32 & 1) == 0) {
code_r0x009503b8:
        Packet$$PutByte(pPVar34,0,(MethodInfo *)0x0);
        pGVar40 = extraout_x0_01;
        pMVar102 = extraout_x1_01;
      }
      else {
        pMVar103 = MobControl_TypeInfo->static_fields->Instance;
        if (((pMVar103 == (MobControl_o *)0x0) ||
            (pSVar55 = (pMVar103->fields).active_combatants,
            pSVar55 == (System_Collections_Generic_Dictionary_string__GameObject__o *)0x0)) ||
           (pUVar56 = (UnityEngine_GameObject_o *)
                      System.Collections.Generic.Dictionary<>$$get_Item
                                (pSVar55,pSVar24,
                                 Method$System.Collections.Generic.Dictionary<string,-GameObject>.ge t_Item()
                                ), pUVar56 == (UnityEngine_GameObject_o *)0x0)) goto LAB_00958368;
        pIVar57 = UnityEngine.GameObject$$GetComponent<object>
                            (pUVar56,Method$UnityEngine.GameObject.GetComponent<SharedCreature>());
        pMVar103 = MobControl_TypeInfo->static_fields->Instance;
        if ((pMVar103 == (MobControl_o *)0x0) ||
           (pSVar27 = (pMVar103->fields).my_claimed_creatures,
           pSVar27 == (System_Collections_Generic_List_string__o *)0x0)) goto LAB_00958368;
        bVar5 = System.Collections.Generic.List<object>$$Contains
                          ((System_Collections_Generic_List_object__o *)pSVar27,
                           (Il2CppObject *)pSVar24,
                           (MethodInfo_F1A0AC *)
                           Method$System.Collections.Generic.List<string>.Contains());
        if (!bVar5) {
          if (pIVar57 == (Il2CppObject *)0x0) goto LAB_00958368;
          if ((*(char *)((long)&pIVar57[10].monitor + 4) == '\0') ||
             (*(int *)&pIVar57[0xb].monitor != 6)) goto code_r0x009503b8;
        }
        Packet$$PutByte(pPVar34,1,(MethodInfo *)0x0);
        pMVar103 = MobControl_TypeInfo->static_fields->Instance;
        if ((pMVar103 == (MobControl_o *)0x0) ||
           (pSVar55 = (pMVar103->fields).active_combatants,
           pSVar55 == (System_Collections_Generic_Dictionary_string__GameObject__o *)0x0))
        goto LAB_00958368;
        pUVar56 = (UnityEngine_GameObject_o *)
                  System.Collections.Generic.Dictionary<>$$get_Item
                            (pSVar55,pSVar24,
                             Method$System.Collections.Generic.Dictionary<string,-GameObject>.get_It em()
                            );
        pMVar103 = MobControl_TypeInfo->static_fields->Instance;
        if ((pMVar103 == (MobControl_o *)0x0) ||
           (pCVar60 = MobControl$$ObjToCreatureStruct(pMVar103,pUVar56,(MethodInfo *)0x0),
           pCVar60 == (CreatureStruct_o *)0x0)) goto LAB_00958368;
        CreatureStruct$$Pack(pCVar60,pPVar34,(MethodInfo *)0x0);
        pMVar102 = (MethodInfo *)0x0;
        Packet$$PutString(pPVar34,pSVar24,(MethodInfo *)0x0);
        if (DAT_028e6c85 == '\0') {
          FUN_0083c778(&GameServerSender_TypeInfo);
          DAT_028e6c85 = '\x01';
        }
        if (pUVar56 == (UnityEngine_GameObject_o *)0x0) goto LAB_00958368;
        pGVar106 = GameServerSender_TypeInfo->static_fields->Instance;
        pUVar59 = UnityEngine.GameObject$$get_transform(pUVar56,(MethodInfo *)0x0);
        if ((pUVar59 == (UnityEngine_Transform_o *)0x0) ||
           (UVar129 = UnityEngine.Transform$$get_position(pUVar59,(MethodInfo *)0x0),
           pGVar106 == (GameServerSender_o *)0x0)) goto LAB_00958368;
        GameServerSender$$PackPosition(__this_01,pPVar34,UVar129,pMVar102);
        pGVar40 = extraout_x0_00;
        pMVar102 = extraout_x1_00;
      }
    }
    pCVar25 = GameServerReceiver$$get_connection(pGVar40,pMVar102);
    if (pCVar25 == (Connection_o *)0x0) goto LAB_00958368;
    iVar17 = 2;
code_r0x00957938:
    Connection$$Send(pCVar25,pPVar34,iVar17,(MethodInfo *)0x0);
    break;
  case 0x43:
    pSVar24 = Packet$$GetString(incoming,(MethodInfo *)0x0);
    uVar6 = Packet$$GetByte(incoming,(MethodInfo *)0x0);
    uVar18 = (uint)uVar6;
    if (uVar6 != 0) {
      do {
        pSVar29 = Packet$$GetString(incoming,(MethodInfo *)0x0);
        uVar6 = Packet$$GetByte(incoming,(MethodInfo *)0x0);
        if (uVar6 == 1) {
          if ((CreatureStruct_TypeInfo->_2).cctor_finished == 0) {
            thunk_FUN_008bc8d8();
          }
          pCVar60 = CreatureStruct$$PacketToCreatureStruct(incoming,(MethodInfo *)0x0);
          pGVar40 = (GameServerReceiver_o *)Packet$$GetString(incoming,(MethodInfo *)0x0);
          UVar129 = GameServerReceiver$$UnpackPosition(pGVar40,incoming,(MethodInfo *)method_04);
          pGVar95 = GameServerInterface_TypeInfo->static_fields->Instance;
          if ((pGVar95 == (GameServerInterface_o *)0x0) ||
             (pSVar39 = (pGVar95->fields).nearby_players,
             pSVar39 == (System_Collections_Generic_Dictionary_string__OnlinePlayer__o *)0x0))
          goto LAB_00958368;
          uVar32 = System.Collections.Generic.Dictionary<>$$ContainsKey
                             (pSVar39,pSVar24,
                              Method$System.Collections.Generic.Dictionary<string,-OnlinePlayer>.Con tainsKey()
                             );
          if ((uVar32 & 1) != 0) {
            pMVar103 = MobControl_TypeInfo->static_fields->Instance;
            if (pMVar103 == (MobControl_o *)0x0) goto LAB_00958368;
            MobControl$$SpawnNetMob
                      (pMVar103,pCVar60,UVar129,(System_String_o *)pGVar40,(MethodInfo *)0x0);
          }
        }
        if (DAT_028e6c85 == '\0') {
          FUN_0083c778(&GameServerSender_TypeInfo);
          DAT_028e6c85 = '\x01';
        }
        pGVar106 = GameServerSender_TypeInfo->static_fields->Instance;
        if ((pGVar106 == (GameServerSender_o *)0x0) ||
           (pSVar27 = (pGVar106->fields).other_players_mobs_that_I_inquired_about,
           pSVar27 == (System_Collections_Generic_List_string__o *)0x0)) goto LAB_00958368;
        method_04 = (Startup_c **)Method$System.Collections.Generic.List<string>.Remove();
        System.Collections.Generic.List<object>$$Remove
                  ((System_Collections_Generic_List_object__o *)pSVar27,(Il2CppObject *)pSVar29,
                   (MethodInfo_F1AFDC *)Method$System.Collections.Generic.List<string>.Remove());
        uVar18 = uVar18 - 1;
      } while (uVar18 != 0);
    }
    break;
  case 0x45:
    pSVar24 = Packet$$GetString(incoming,(MethodInfo *)0x0);
    pMVar103 = MobControl_TypeInfo->static_fields->Instance;
    if ((pMVar103 != (MobControl_o *)0x0) &&
       (pSVar55 = (pMVar103->fields).active_combatants,
       pSVar55 != (System_Collections_Generic_Dictionary_string__GameObject__o *)0x0)) {
      uVar32 = System.Collections.Generic.Dictionary<>$$ContainsKey
                         (pSVar55,pSVar24,
                          Method$System.Collections.Generic.Dictionary<string,-GameObject>.ContainsK ey()
                         );
      if ((uVar32 & 1) == 0) {
        return;
      }
      pMVar103 = MobControl_TypeInfo->static_fields->Instance;
      if ((pMVar103 != (MobControl_o *)0x0) &&
         (pSVar55 = (pMVar103->fields).active_combatants,
         pSVar55 != (System_Collections_Generic_Dictionary_string__GameObject__o *)0x0)) {
        pUVar74 = (UnityEngine_Object_o *)
                  System.Collections.Generic.Dictionary<>$$get_Item
                            (pSVar55,pSVar24,
                             Method$System.Collections.Generic.Dictionary<string,-GameObject>.get_It em()
                            );
        if ((UnityEngine.Object_TypeInfo->_2).cctor_finished == 0) {
          thunk_FUN_008bc8d8(UnityEngine.Object_TypeInfo);
        }
        bVar5 = UnityEngine.Object$$op_Equality
                          (pUVar74,(UnityEngine_Object_o *)0x0,(MethodInfo *)0x0);
        if (bVar5) {
          return;
        }
        pMVar103 = MobControl_TypeInfo->static_fields->Instance;
        if ((((pMVar103 != (MobControl_o *)0x0) &&
             (pSVar55 = (pMVar103->fields).active_combatants,
             pSVar55 != (System_Collections_Generic_Dictionary_string__GameObject__o *)0x0)) &&
            (pUVar56 = (UnityEngine_GameObject_o *)
                       System.Collections.Generic.Dictionary<>$$get_Item
                                 (pSVar55,pSVar24,
                                  Method$System.Collections.Generic.Dictionary<string,-GameObject>.g et_Item()
                                 ), pUVar56 != (UnityEngine_GameObject_o *)0x0)) &&
           (pUVar59 = UnityEngine.GameObject$$get_transform(pUVar56,(MethodInfo *)0x0),
           pUVar59 != (UnityEngine_Transform_o *)0x0)) {
          UVar129 = UnityEngine.Transform$$get_position(pUVar59,(MethodInfo *)0x0);
          pUVar59 = UnityEngine.GameObject$$get_transform(pUVar56,(MethodInfo *)0x0);
          if (pUVar59 != (UnityEngine_Transform_o *)0x0) {
            UVar131 = UnityEngine.Transform$$get_rotation(pUVar59,(MethodInfo *)0x0);
            pMVar103 = MobControl_TypeInfo->static_fields->Instance;
            if (pMVar103 != (MobControl_o *)0x0) {
              pCVar60 = MobControl$$ObjToCreatureStruct(pMVar103,pUVar56,(MethodInfo *)0x0);
              pSVar26 = (SharedCreature_o *)
                        UnityEngine.GameObject$$GetComponent<object>
                                  (pUVar56,
                                   Method$UnityEngine.GameObject.GetComponent<SharedCreature>());
              if (pSVar26 != (SharedCreature_o *)0x0) {
                SharedCreature$$Delete(pSVar26,(MethodInfo *)0x0);
                pCVar62 = (Combatant_o *)
                          UnityEngine.GameObject$$GetComponent<object>
                                    (pUVar56,Method$UnityEngine.GameObject.GetComponent<Combatant>()
                                    );
                if (pCVar62 != (Combatant_o *)0x0) {
                  Combatant$$Delete(pCVar62,(MethodInfo *)0x0);
                  pUVar74 = (UnityEngine_Object_o *)
                            UnityEngine.GameObject$$get_gameObject(pUVar56,(MethodInfo *)0x0);
                  if ((UnityEngine.Object_TypeInfo->_2).cctor_finished == 0) {
                    thunk_FUN_008bc8d8(UnityEngine.Object_TypeInfo);
                  }
                  UnityEngine.Object$$Destroy(pUVar74,(MethodInfo *)0x0);
                  pMVar103 = MobControl_TypeInfo->static_fields->Instance;
                  if (((pMVar103 != (MobControl_o *)0x0) &&
                      (pUVar56 = MobControl$$SpawnLocalMob
                                           (pMVar103,pCVar60,UVar129,(MethodInfo *)0x0),
                      pUVar56 != (UnityEngine_GameObject_o *)0x0)) &&
                     (pUVar59 = UnityEngine.GameObject$$get_transform(pUVar56,(MethodInfo *)0x0),
                     pUVar59 != (UnityEngine_Transform_o *)0x0)) {
                    UnityEngine.Transform$$set_rotation(pUVar59,UVar131,(MethodInfo *)0x0);
                    return;
                  }
                }
              }
            }
          }
        }
      }
    }
    goto LAB_00958368;
  case 0x46:
    pSVar24 = Packet$$GetString(incoming,(MethodInfo *)0x0);
    pMVar103 = MobControl_TypeInfo->static_fields->Instance;
    if ((pMVar103 == (MobControl_o *)0x0) ||
       (pSVar55 = (pMVar103->fields).active_combatants,
       pSVar55 == (System_Collections_Generic_Dictionary_string__GameObject__o *)0x0))
    goto LAB_00958368;
    uVar32 = System.Collections.Generic.Dictionary<>$$ContainsKey
                       (pSVar55,pSVar24,
                        Method$System.Collections.Generic.Dictionary<string,-GameObject>.ContainsKey ()
                       );
    if ((uVar32 & 1) != 0) {
      pMVar103 = MobControl_TypeInfo->static_fields->Instance;
      if (((pMVar103 == (MobControl_o *)0x0) ||
          (pSVar55 = (pMVar103->fields).active_combatants,
          pSVar55 == (System_Collections_Generic_Dictionary_string__GameObject__o *)0x0)) ||
         ((pUVar56 = (UnityEngine_GameObject_o *)
                     System.Collections.Generic.Dictionary<>$$get_Item
                               (pSVar55,pSVar24,
                                Method$System.Collections.Generic.Dictionary<string,-GameObject>.get _Item()
                               ), pUVar56 == (UnityEngine_GameObject_o *)0x0 ||
          (pSVar26 = (SharedCreature_o *)
                     UnityEngine.GameObject$$GetComponent<object>
                               (pUVar56,Method$UnityEngine.GameObject.GetComponent<SharedCreature>()
                               ), pSVar26 == (SharedCreature_o *)0x0)))) goto LAB_00958368;
      SharedCreature$$VisuallyAttack(pSVar26,(MethodInfo *)0x0);
    }
    break;
  case 0x47:
    pSVar24 = Packet$$GetString(incoming,(MethodInfo *)0x0);
    iVar17 = Packet$$GetLong(incoming,(MethodInfo *)0x0);
    iVar21 = Packet$$GetLong(incoming,(MethodInfo *)0x0);
    uVar6 = Packet$$GetByte(incoming,(MethodInfo *)0x0);
    uVar8 = Packet$$GetByte(incoming,(MethodInfo *)0x0);
    uVar7 = Packet$$GetByte(incoming,(MethodInfo *)0x0);
    pSVar29 = Packet$$GetString(incoming,(MethodInfo *)0x0);
    pPVar64 = PlayerData_TypeInfo->static_fields->Instance;
    if (pPVar64 != (PlayerData_o *)0x0) {
      pSVar63 = PlayerData$$GetGlobalString(pPVar64,StringLiteral_14006,(MethodInfo *)0x0);
      bVar5 = System.String$$op_Equality(pSVar24,pSVar63,(MethodInfo *)0x0);
      pMVar103 = MobControl_TypeInfo->static_fields->Instance;
      pSVar63 = StringLiteral_5465;
      if (!bVar5) {
        pSVar63 = pSVar24;
      }
      if ((pMVar103 != (MobControl_o *)0x0) &&
         (pSVar55 = (pMVar103->fields).active_combatants,
         pSVar55 != (System_Collections_Generic_Dictionary_string__GameObject__o *)0x0)) {
        uVar32 = System.Collections.Generic.Dictionary<>$$ContainsKey
                           (pSVar55,pSVar63,
                            Method$System.Collections.Generic.Dictionary<string,-GameObject>.Contain sKey()
                           );
        if ((uVar32 & 1) == 0) {
          return;
        }
        pMVar103 = MobControl_TypeInfo->static_fields->Instance;
        if ((pMVar103 != (MobControl_o *)0x0) &&
           (pSVar55 = (pMVar103->fields).active_combatants,
           pSVar55 != (System_Collections_Generic_Dictionary_string__GameObject__o *)0x0)) {
          uVar32 = System.Collections.Generic.Dictionary<>$$ContainsKey
                             (pSVar55,pSVar29,
                              Method$System.Collections.Generic.Dictionary<string,-GameObject>.Conta insKey()
                             );
          if ((uVar32 & 1) == 0) {
            pUVar56 = (UnityEngine_GameObject_o *)0x0;
          }
          else {
            pMVar103 = MobControl_TypeInfo->static_fields->Instance;
            if ((pMVar103 == (MobControl_o *)0x0) ||
               (pSVar55 = (pMVar103->fields).active_combatants,
               pSVar55 == (System_Collections_Generic_Dictionary_string__GameObject__o *)0x0))
            goto LAB_00958368;
            pUVar56 = (UnityEngine_GameObject_o *)
                      System.Collections.Generic.Dictionary<>$$get_Item
                                (pSVar55,pSVar29,
                                 Method$System.Collections.Generic.Dictionary<string,-GameObject>.ge t_Item()
                                );
          }
          pMVar103 = MobControl_TypeInfo->static_fields->Instance;
          if ((((pMVar103 != (MobControl_o *)0x0) &&
               (pSVar55 = (pMVar103->fields).active_combatants,
               pSVar55 != (System_Collections_Generic_Dictionary_string__GameObject__o *)0x0)) &&
              (pUVar86 = (UnityEngine_GameObject_o *)
                         System.Collections.Generic.Dictionary<>$$get_Item
                                   (pSVar55,pSVar63,
                                    Method$System.Collections.Generic.Dictionary<string,-GameObject> .get_Item()
                                   ), pUVar86 != (UnityEngine_GameObject_o *)0x0)) &&
             (pCVar62 = (Combatant_o *)
                        UnityEngine.GameObject$$GetComponent<object>
                                  (pUVar86,Method$UnityEngine.GameObject.GetComponent<Combatant>()),
             pCVar62 != (Combatant_o *)0x0)) {
            Combatant$$WasHit(pCVar62,iVar17,pUVar56,uVar8 == 1,uVar7 == 1,(uint)uVar6,false,iVar21,
                              (MethodInfo *)0x0);
            return;
          }
        }
      }
    }
    goto LAB_00958368;
  case 0x48:
    pSVar24 = Packet$$GetString(incoming,(MethodInfo *)0x0);
    iVar10 = Packet$$GetShort(incoming,(MethodInfo *)0x0);
    iVar11 = Packet$$GetShort(incoming,(MethodInfo *)0x0);
    pSVar29 = Packet$$GetString(incoming,(MethodInfo *)0x0);
    iVar12 = Packet$$GetShort(incoming,(MethodInfo *)0x0);
    iVar13 = Packet$$GetShort(incoming,(MethodInfo *)0x0);
    iVar14 = Packet$$GetShort(incoming,(MethodInfo *)0x0);
    iVar15 = Packet$$GetShort(incoming,(MethodInfo *)0x0);
    iVar16 = Packet$$GetShort(incoming,(MethodInfo *)0x0);
    pSVar63 = Packet$$GetString(incoming,(MethodInfo *)0x0);
    uVar6 = Packet$$GetByte(incoming,(MethodInfo *)0x0);
    uVar8 = Packet$$GetByte(incoming,(MethodInfo *)0x0);
    pIVar68 = InventoryItem$$UnpackFromWeb(incoming,(MethodInfo *)0x0);
    bVar5 = System.String$$op_Equality
                      (pSVar29,(System_String_o *)StringLiteral_1.rgctx_data,(MethodInfo *)0x0);
    pPVar64 = PlayerData_TypeInfo->static_fields->Instance;
    if (pPVar64 == (PlayerData_o *)0x0) goto LAB_00958368;
    pSVar65 = PlayerData$$GetGlobalString(pPVar64,StringLiteral_14006,(MethodInfo *)0x0);
    bVar9 = System.String$$op_Equality(pSVar63,pSVar65,(MethodInfo *)0x0);
    pMVar103 = MobControl_TypeInfo->static_fields->Instance;
    pSVar65 = StringLiteral_5465;
    if (!bVar9) {
      pSVar65 = pSVar63;
    }
    if ((pMVar103 == (MobControl_o *)0x0) ||
       (pSVar55 = (pMVar103->fields).active_combatants,
       pSVar55 == (System_Collections_Generic_Dictionary_string__GameObject__o *)0x0))
    goto LAB_00958368;
    uVar32 = System.Collections.Generic.Dictionary<>$$ContainsKey
                       (pSVar55,pSVar24,
                        Method$System.Collections.Generic.Dictionary<string,-GameObject>.ContainsKey ()
                       );
    if ((uVar32 & 1) == 0) {
      pGVar90 = GameServerConnector_TypeInfo->static_fields->Instance;
      if (pGVar90 != (GameServerConnector_o *)0x0) {
        if (((pGVar90->fields).is_host_cached &
            ((((iVar13 != 0 || iVar12 != 0) || iVar14 != 0) || iVar15 != 0) | (bVar5 ^ 0xffU) & 1))
            == 0) {
          return;
        }
        if ((QuestControl_TypeInfo->_2).cctor_finished == 0) {
          thunk_FUN_008bc8d8();
        }
        pQVar78 = QuestControl_TypeInfo->static_fields->Instance;
        if (pQVar78 != (QuestControl_o *)0x0) {
          QuestControl$$TryNoteQuestMobKilled
                    (pQVar78,pSVar29,(int)iVar12,(int)iVar13,(int)iVar14,(int)iVar15,pIVar68,pSVar24
                     ,(MethodInfo *)0x0);
          pQVar78 = QuestControl_TypeInfo->static_fields->Instance;
          if (pQVar78 != (QuestControl_o *)0x0) {
            QuestControl$$CheckIfAllQuestMobsKilled
                      (pQVar78,pSVar29,(int)iVar12,(int)iVar13,(int)iVar14,(int)iVar15,pIVar68,
                       pSVar24,(MethodInfo *)0x0);
            return;
          }
        }
      }
      goto LAB_00958368;
    }
    pMVar103 = MobControl_TypeInfo->static_fields->Instance;
    if ((pMVar103 == (MobControl_o *)0x0) ||
       (pSVar55 = (pMVar103->fields).active_combatants,
       pSVar55 == (System_Collections_Generic_Dictionary_string__GameObject__o *)0x0))
    goto LAB_00958368;
    uVar32 = System.Collections.Generic.Dictionary<>$$ContainsKey
                       (pSVar55,pSVar65,
                        Method$System.Collections.Generic.Dictionary<string,-GameObject>.ContainsKey ()
                       );
    if ((uVar32 & 1) == 0) {
      pUVar56 = (UnityEngine_GameObject_o *)0x0;
    }
    else {
      pMVar103 = MobControl_TypeInfo->static_fields->Instance;
      if ((pMVar103 == (MobControl_o *)0x0) ||
         (pSVar55 = (pMVar103->fields).active_combatants,
         pSVar55 == (System_Collections_Generic_Dictionary_string__GameObject__o *)0x0))
      goto LAB_00958368;
      pUVar56 = (UnityEngine_GameObject_o *)
                System.Collections.Generic.Dictionary<>$$get_Item
                          (pSVar55,pSVar65,
                           Method$System.Collections.Generic.Dictionary<string,-GameObject>.get_Item ()
                          );
    }
    pMVar103 = MobControl_TypeInfo->static_fields->Instance;
    if ((pMVar103 == (MobControl_o *)0x0) ||
       (pSVar55 = (pMVar103->fields).active_combatants,
       pSVar55 == (System_Collections_Generic_Dictionary_string__GameObject__o *)0x0))
    goto LAB_00958368;
    pUVar86 = (UnityEngine_GameObject_o *)
              System.Collections.Generic.Dictionary<>$$get_Item
                        (pSVar55,pSVar24,
                         Method$System.Collections.Generic.Dictionary<string,-GameObject>.get_Item()
                        );
    if (uVar6 == 1) {
      if (pUVar86 == (UnityEngine_GameObject_o *)0x0) goto LAB_00958368;
      pCVar62 = (Combatant_o *)
                UnityEngine.GameObject$$GetComponent<object>
                          (pUVar86,Method$UnityEngine.GameObject.GetComponent<Combatant>());
      pPVar30 = PerkControl_TypeInfo->static_fields->Instance;
      if ((pPVar30 == (PerkControl_o *)0x0) || (pCVar62 == (Combatant_o *)0x0)) goto LAB_00958368;
      ppUVar113 = &(pPVar30->fields).prefab_darksword_kill;
code_r0x00957138:
      Combatant$$BeginDarkswordParticles(pCVar62,*ppUVar113,(MethodInfo *)0x0);
    }
    else {
      if (uVar8 == 1) {
        if (pUVar86 == (UnityEngine_GameObject_o *)0x0) goto LAB_00958368;
        pCVar62 = (Combatant_o *)
                  UnityEngine.GameObject$$GetComponent<object>
                            (pUVar86,Method$UnityEngine.GameObject.GetComponent<Combatant>());
        pPVar30 = PerkControl_TypeInfo->static_fields->Instance;
        if ((pPVar30 == (PerkControl_o *)0x0) || (pCVar62 == (Combatant_o *)0x0)) goto LAB_00958368;
        ppUVar113 = &(pPVar30->fields).prefab_aether_banish;
        goto code_r0x00957138;
      }
      if (pUVar86 == (UnityEngine_GameObject_o *)0x0) goto LAB_00958368;
    }
    pCVar62 = (Combatant_o *)
              UnityEngine.GameObject$$GetComponent<object>
                        (pUVar86,Method$UnityEngine.GameObject.GetComponent<Combatant>());
    if (pCVar62 == (Combatant_o *)0x0) goto LAB_00958368;
    Combatant$$Die(pCVar62,pUVar56,(float)(int)iVar10 / 10.0,(float)(int)iVar11 / 10.0,(int)iVar16,
                   (MethodInfo *)0x0);
    break;
  case 0x4a:
    pSVar24 = Packet$$GetString(incoming,(MethodInfo *)0x0);
    iVar17 = Packet$$GetLong(incoming,(MethodInfo *)0x0);
    iVar21 = Packet$$GetLong(incoming,(MethodInfo *)0x0);
    iVar22 = Packet$$GetLong(incoming,(MethodInfo *)0x0);
    iVar23 = Packet$$GetLong(incoming,(MethodInfo *)0x0);
    pMVar103 = MobControl_TypeInfo->static_fields->Instance;
    if ((pMVar103 != (MobControl_o *)0x0) &&
       (pSVar55 = (pMVar103->fields).active_combatants,
       pSVar55 != (System_Collections_Generic_Dictionary_string__GameObject__o *)0x0)) {
      uVar32 = System.Collections.Generic.Dictionary<>$$ContainsKey
                         (pSVar55,pSVar24,
                          Method$System.Collections.Generic.Dictionary<string,-GameObject>.ContainsK ey()
                         );
      if ((uVar32 & 1) == 0) {
        return;
      }
      pMVar103 = MobControl_TypeInfo->static_fields->Instance;
      if (((pMVar103 != (MobControl_o *)0x0) &&
          (pSVar55 = (pMVar103->fields).active_combatants,
          pSVar55 != (System_Collections_Generic_Dictionary_string__GameObject__o *)0x0)) &&
         ((pUVar56 = (UnityEngine_GameObject_o *)
                     System.Collections.Generic.Dictionary<>$$get_Item
                               (pSVar55,pSVar24,
                                Method$System.Collections.Generic.Dictionary<string,-GameObject>.get _Item()
                               ), pUVar56 != (UnityEngine_GameObject_o *)0x0 &&
          (pIVar57 = UnityEngine.GameObject$$GetComponent<object>
                               (pUVar56,Method$UnityEngine.GameObject.GetComponent<SharedCreature>()
                               ), pIVar57 != (Il2CppObject *)0x0)))) {
        iVar19 = *(int *)&pIVar57[9].monitor;
        pIVar57 = UnityEngine.GameObject$$GetComponent<object>
                            (pUVar56,Method$UnityEngine.GameObject.GetComponent<SharedCreature>());
        if (pIVar57 != (Il2CppObject *)0x0) {
          *(int32_t *)&pIVar57[9].monitor = iVar17;
          pIVar57 = UnityEngine.GameObject$$GetComponent<object>
                              (pUVar56,Method$UnityEngine.GameObject.GetComponent<Combatant>());
          if (pIVar57 != (Il2CppObject *)0x0) {
            *(int32_t *)&pIVar57[2].monitor = iVar21;
            pIVar57 = UnityEngine.GameObject$$GetComponent<object>
                                (pUVar56,Method$UnityEngine.GameObject.GetComponent<Combatant>());
            if (pIVar57 != (Il2CppObject *)0x0) {
              *(float *)((long)&pIVar57[2].klass + 4) = (float)iVar22;
              pIVar57 = UnityEngine.GameObject$$GetComponent<object>
                                  (pUVar56,
                                   Method$UnityEngine.GameObject.GetComponent<SharedCreature>());
              if (pIVar57 != (Il2CppObject *)0x0) {
                *(int32_t *)&pIVar57[10].klass = iVar23;
                if (iVar17 == iVar19) {
                  return;
                }
                pSVar26 = (SharedCreature_o *)
                          UnityEngine.GameObject$$GetComponent<object>
                                    (pUVar56,
                                     Method$UnityEngine.GameObject.GetComponent<SharedCreature>());
                if (pSVar26 != (SharedCreature_o *)0x0) {
                  SharedCreature$$ShowLevelupParticles(pSVar26,true,0.0,(MethodInfo *)0x0);
                  pGVar95 = GameServerInterface_TypeInfo->static_fields->Instance;
                  if ((pGVar95 != (GameServerInterface_o *)0x0) &&
                     (pSVar39 = (pGVar95->fields).nearby_players,
                     pSVar39 != (System_Collections_Generic_Dictionary_string__OnlinePlayer__o *)0x0
                     )) {
                    uVar32 = System.Collections.Generic.Dictionary<>$$ContainsKey
                                       (pSVar39,pSVar24,
                                        Method$System.Collections.Generic.Dictionary<string,-OnlineP layer>.ContainsKey()
                                       );
                    if ((uVar32 & 1) == 0) {
                      pSVar26 = (SharedCreature_o *)
                                UnityEngine.GameObject$$GetComponent<object>
                                          (pUVar56,
                                           Method$UnityEngine.GameObject.GetComponent<SharedCreature >()
                                          );
                      if (pSVar26 != (SharedCreature_o *)0x0) {
                        SharedCreature$$RedrawLevelText(pSVar26,(MethodInfo *)0x0);
                        return;
                      }
                    }
                    else {
                      pGVar95 = GameServerInterface_TypeInfo->static_fields->Instance;
                      if (((pGVar95 != (GameServerInterface_o *)0x0) &&
                          (pSVar39 = (pGVar95->fields).nearby_players,
                          pSVar39 !=
                          (System_Collections_Generic_Dictionary_string__OnlinePlayer__o *)0x0)) &&
                         (lVar61 = System.Collections.Generic.Dictionary<>$$get_Item
                                             (pSVar39,pSVar24,
                                              Method$System.Collections.Generic.Dictionary<string,-O nlinePlayer>.get_Item()
                                             ), lVar61 != 0)) {
                        pUVar74 = *(UnityEngine_Object_o **)(lVar61 + 0x10);
                        if ((UnityEngine.Object_TypeInfo->_2).cctor_finished == 0) {
                          thunk_FUN_008bc8d8(UnityEngine.Object_TypeInfo);
                        }
                        bVar5 = UnityEngine.Object$$op_Inequality
                                          (pUVar74,(UnityEngine_Object_o *)0x0,(MethodInfo *)0x0);
                        if (!bVar5) {
                          return;
                        }
                        pGVar95 = GameServerInterface_TypeInfo->static_fields->Instance;
                        if (((pGVar95 != (GameServerInterface_o *)0x0) &&
                            (pSVar39 = (pGVar95->fields).nearby_players,
                            pSVar39 !=
                            (System_Collections_Generic_Dictionary_string__OnlinePlayer__o *)0x0))
                           && ((lVar61 = System.Collections.Generic.Dictionary<>$$get_Item
                                                   (pSVar39,pSVar24,
                                                                                                        
                                                  Method$System.Collections.Generic.Dictionary<strin g,-OnlinePlayer>.get_Item()
                                                  ), lVar61 != 0 &&
                               ((*(UnityEngine_GameObject_o **)(lVar61 + 0x10) !=
                                 (UnityEngine_GameObject_o *)0x0 &&
                                (pSVar26 = (SharedCreature_o *)
                                           UnityEngine.GameObject$$GetComponent<object>
                                                     (*(UnityEngine_GameObject_o **)(lVar61 + 0x10),
                                                                                                            
                                                  Method$UnityEngine.GameObject.GetComponent<SharedC reature>()
                                                  ), pSVar26 != (SharedCreature_o *)0x0)))))) {
                          SharedCreature$$RedrawMultiplayerOverhead
                                    (pSVar26,pSVar24,iVar17,(MethodInfo *)0x0);
                          return;
                        }
                      }
                    }
                  }
                }
              }
            }
          }
        }
      }
    }
    goto LAB_00958368;
  case 0x4b:
    pSVar24 = Packet$$GetString(incoming,(MethodInfo *)0x0);
    iVar17 = Packet$$GetLong(incoming,(MethodInfo *)0x0);
    pPVar64 = PlayerData_TypeInfo->static_fields->Instance;
    if (pPVar64 != (PlayerData_o *)0x0) {
      pSVar29 = PlayerData$$GetGlobalString(pPVar64,StringLiteral_14006,(MethodInfo *)0x0);
      bVar5 = System.String$$op_Equality(pSVar24,pSVar29,(MethodInfo *)0x0);
      pMVar103 = MobControl_TypeInfo->static_fields->Instance;
      pSVar29 = StringLiteral_5465;
      if (!bVar5) {
        pSVar29 = pSVar24;
      }
      if ((pMVar103 != (MobControl_o *)0x0) &&
         (pSVar55 = (pMVar103->fields).active_combatants,
         pSVar55 != (System_Collections_Generic_Dictionary_string__GameObject__o *)0x0)) {
        uVar32 = System.Collections.Generic.Dictionary<>$$ContainsKey
                           (pSVar55,pSVar29,
                            Method$System.Collections.Generic.Dictionary<string,-GameObject>.Contain sKey()
                           );
        if ((uVar32 & 1) == 0) {
          return;
        }
        pMVar103 = MobControl_TypeInfo->static_fields->Instance;
        if ((((pMVar103 != (MobControl_o *)0x0) &&
             (pSVar55 = (pMVar103->fields).active_combatants,
             pSVar55 != (System_Collections_Generic_Dictionary_string__GameObject__o *)0x0)) &&
            (pUVar56 = (UnityEngine_GameObject_o *)
                       System.Collections.Generic.Dictionary<>$$get_Item
                                 (pSVar55,pSVar29,
                                  Method$System.Collections.Generic.Dictionary<string,-GameObject>.g et_Item()
                                 ), pUVar56 != (UnityEngine_GameObject_o *)0x0)) &&
           (pCVar62 = (Combatant_o *)
                      UnityEngine.GameObject$$GetComponent<object>
                                (pUVar56,Method$UnityEngine.GameObject.GetComponent<Combatant>()),
           pCVar62 != (Combatant_o *)0x0)) {
          Combatant$$IncreaseHp(pCVar62,iVar17,(MethodInfo *)0x0);
          return;
        }
      }
    }
    goto LAB_00958368;
  case 0x4c:
    pGVar40 = (GameServerReceiver_o *)Packet$$GetString(incoming,(MethodInfo *)0x0);
    UVar129 = GameServerReceiver$$UnpackPosition(pGVar40,incoming,(MethodInfo *)method_04);
    if ((GameController_TypeInfo->_2).cctor_finished == 0) {
      thunk_FUN_008bc8d8();
    }
    pGVar93 = GameController_TypeInfo->static_fields->Instance;
    if (pGVar93 == (GameController_o *)0x0) goto LAB_00958368;
    GameController$$showOverheadNotif
              (pGVar93,(System_String_o *)pGVar40,UVar129,false,false,(MethodInfo *)0x0);
    break;
  case 0x4e:
    pSVar24 = Packet$$GetString(incoming,(MethodInfo *)0x0);
    pIVar68 = InventoryItem$$UnpackFromWeb(incoming,(MethodInfo *)0x0);
    pIVar69 = InventoryItem$$UnpackFromWeb(incoming,(MethodInfo *)0x0);
    pIVar70 = InventoryItem$$UnpackFromWeb(incoming,(MethodInfo *)0x0);
    pMVar103 = MobControl_TypeInfo->static_fields->Instance;
    if ((pMVar103 == (MobControl_o *)0x0) ||
       (pSVar55 = (pMVar103->fields).active_combatants,
       pSVar55 == (System_Collections_Generic_Dictionary_string__GameObject__o *)0x0))
    goto LAB_00958368;
    uVar32 = System.Collections.Generic.Dictionary<>$$ContainsKey
                       (pSVar55,pSVar24,
                        Method$System.Collections.Generic.Dictionary<string,-GameObject>.ContainsKey ()
                       );
    if ((uVar32 & 1) != 0) {
      pMVar103 = MobControl_TypeInfo->static_fields->Instance;
      if ((pMVar103 == (MobControl_o *)0x0) ||
         (pSVar55 = (pMVar103->fields).active_combatants,
         pSVar55 == (System_Collections_Generic_Dictionary_string__GameObject__o *)0x0))
      goto LAB_00958368;
      pUVar74 = (UnityEngine_Object_o *)
                System.Collections.Generic.Dictionary<>$$get_Item
                          (pSVar55,pSVar24,
                           Method$System.Collections.Generic.Dictionary<string,-GameObject>.get_Item ()
                          );
      if ((UnityEngine.Object_TypeInfo->_2).cctor_finished == 0) {
        thunk_FUN_008bc8d8(UnityEngine.Object_TypeInfo);
      }
      bVar5 = UnityEngine.Object$$op_Inequality
                        (pUVar74,(UnityEngine_Object_o *)0x0,(MethodInfo *)0x0);
      if (bVar5) {
        if ((pUVar74 == (UnityEngine_Object_o *)0x0) ||
           (pIVar57 = UnityEngine.GameObject$$GetComponent<object>
                                ((UnityEngine_GameObject_o *)pUVar74,
                                 Method$UnityEngine.GameObject.GetComponent<Combatant>()),
           pIVar57 == (Il2CppObject *)0x0)) goto LAB_00958368;
        if (*(int *)((long)&pIVar57[3].monitor + 4) == 3) {
          pUVar56 = UnityEngine.GameObject$$get_gameObject
                              ((UnityEngine_GameObject_o *)pUVar74,(MethodInfo *)0x0);
          if ((pUVar56 == (UnityEngine_GameObject_o *)0x0) ||
             (pSVar26 = (SharedCreature_o *)
                        UnityEngine.GameObject$$GetComponent<object>
                                  (pUVar56,
                                   Method$UnityEngine.GameObject.GetComponent<SharedCreature>()),
             pSVar26 == (SharedCreature_o *)0x0)) goto LAB_00958368;
          if (((pSVar26->fields).is_local_mob == false) &&
             ((pSVar26->fields).cached_brain_type == 6)) {
            ppIVar71 = &(pSVar26->fields).hat_;
            *ppIVar71 = pIVar68;
            thunk_FUN_008c6b48(ppIVar71,pIVar68);
            ppIVar71 = &(pSVar26->fields).body_;
            *ppIVar71 = pIVar69;
            thunk_FUN_008c6b48(ppIVar71,pIVar69);
            ppIVar71 = &(pSVar26->fields).hand_;
            *ppIVar71 = pIVar70;
            thunk_FUN_008c6b48(ppIVar71,pIVar70);
            SharedCreature$$OnEquipmentChanged(pSVar26,(MethodInfo *)0x0);
          }
        }
      }
    }
    break;
  case 0x4f:
    pSVar24 = Packet$$GetString(incoming,(MethodInfo *)0x0);
    pSVar29 = Packet$$GetString(incoming,(MethodInfo *)0x0);
    pMVar103 = MobControl_TypeInfo->static_fields->Instance;
    if ((pMVar103 != (MobControl_o *)0x0) &&
       (pSVar55 = (pMVar103->fields).active_combatants,
       pSVar55 != (System_Collections_Generic_Dictionary_string__GameObject__o *)0x0)) {
      uVar32 = System.Collections.Generic.Dictionary<>$$ContainsKey
                         (pSVar55,pSVar24,
                          Method$System.Collections.Generic.Dictionary<string,-GameObject>.ContainsK ey()
                         );
      if ((uVar32 & 1) == 0) {
        return;
      }
      pMVar103 = MobControl_TypeInfo->static_fields->Instance;
      if ((pMVar103 != (MobControl_o *)0x0) &&
         (pSVar55 = (pMVar103->fields).active_combatants,
         pSVar55 != (System_Collections_Generic_Dictionary_string__GameObject__o *)0x0)) {
        pUVar74 = (UnityEngine_Object_o *)
                  System.Collections.Generic.Dictionary<>$$get_Item
                            (pSVar55,pSVar24,
                             Method$System.Collections.Generic.Dictionary<string,-GameObject>.get_It em()
                            );
        if ((UnityEngine.Object_TypeInfo->_2).cctor_finished == 0) {
          thunk_FUN_008bc8d8(UnityEngine.Object_TypeInfo);
        }
        bVar5 = UnityEngine.Object$$op_Inequality
                          (pUVar74,(UnityEngine_Object_o *)0x0,(MethodInfo *)0x0);
        if (!bVar5) {
          return;
        }
        if ((pUVar74 != (UnityEngine_Object_o *)0x0) &&
           (pIVar57 = UnityEngine.GameObject$$GetComponent<object>
                                ((UnityEngine_GameObject_o *)pUVar74,
                                 Method$UnityEngine.GameObject.GetComponent<Combatant>()),
           pIVar57 != (Il2CppObject *)0x0)) {
          if (*(int *)((long)&pIVar57[3].monitor + 4) != 3) {
            return;
          }
          pUVar56 = UnityEngine.GameObject$$get_gameObject
                              ((UnityEngine_GameObject_o *)pUVar74,(MethodInfo *)0x0);
          if ((pUVar56 != (UnityEngine_GameObject_o *)0x0) &&
             (pSVar26 = (SharedCreature_o *)
                        UnityEngine.GameObject$$GetComponent<object>
                                  (pUVar56,
                                   Method$UnityEngine.GameObject.GetComponent<SharedCreature>()),
             pSVar26 != (SharedCreature_o *)0x0)) {
            if ((pSVar26->fields).is_local_mob != false) {
              return;
            }
            if ((pSVar26->fields).cached_brain_type != 6) {
              return;
            }
            pLVar91 = (pSVar26->fields).myCreatureModel;
            if (((pLVar91 != (LiteModel_o *)0x0) &&
                (pCVar92 = (pLVar91->fields).original, pCVar92 != (CreatureModel_o *)0x0)) &&
               (pSVar27 = (pCVar92->fields).creatures_that_made_me,
               pSVar27 != (System_Collections_Generic_List_string__o *)0x0)) {
              pMVar103 = MobControl_TypeInfo->static_fields->Instance;
              pSVar24 = (System_String_o *)
                        System.Collections.Generic.List<object>$$get_Item
                                  ((System_Collections_Generic_List_object__o *)pSVar27,0,
                                   Method$System.Collections.Generic.List<string>.get_Item());
              pLVar91 = (pSVar26->fields).myCreatureModel;
              if (((pLVar91 != (LiteModel_o *)0x0) &&
                  (pCVar92 = (pLVar91->fields).original, pCVar92 != (CreatureModel_o *)0x0)) &&
                 (pSVar27 = (pCVar92->fields).creatures_that_made_me,
                 pSVar27 != (System_Collections_Generic_List_string__o *)0x0)) {
                pSVar63 = (System_String_o *)
                          System.Collections.Generic.List<object>$$get_Item
                                    ((System_Collections_Generic_List_object__o *)pSVar27,1,
                                     Method$System.Collections.Generic.List<string>.get_Item());
                pSVar24 = System.String$$Concat(pSVar24,pSVar63,(MethodInfo *)0x0);
                if (pMVar103 != (MobControl_o *)0x0) {
                  creature_name_col =
                       MobControl$$GetOverheadNameColor(pMVar103,pSVar24,(MethodInfo *)0x0);
                  SharedCreature$$AssignOverheadName
                            (pSVar26,pSVar29,creature_name_col,(MethodInfo *)0x0);
                  pUVar74 = (UnityEngine_Object_o *)(pSVar26->fields).levelDisplay;
                  if ((UnityEngine.Object_TypeInfo->_2).cctor_finished == 0) {
                    thunk_FUN_008bc8d8();
                  }
                  bVar5 = UnityEngine.Object$$op_Inequality
                                    (pUVar74,(UnityEngine_Object_o *)0x0,(MethodInfo *)0x0);
                  if (bVar5) {
                    if ((GameController_TypeInfo->_2).cctor_finished == 0) {
                      thunk_FUN_008bc8d8();
                    }
                    pGVar93 = GameController_TypeInfo->static_fields->Instance;
                    if ((pGVar93 == (GameController_o *)0x0) ||
                       (__this_00 = (pGVar93->fields).possible_destroy,
                       __this_00 == (System_Collections_Generic_List_GameObject__o *)0x0))
                    goto LAB_00958368;
                    System.Collections.Generic.List<object>$$Remove
                              ((System_Collections_Generic_List_object__o *)__this_00,
                               (Il2CppObject *)(pSVar26->fields).levelDisplay,
                               Method$System.Collections.Generic.List<GameObject>.Remove());
                    pUVar74 = (UnityEngine_Object_o *)(pSVar26->fields).levelDisplay;
                    if ((UnityEngine.Object_TypeInfo->_2).cctor_finished == 0) {
                      thunk_FUN_008bc8d8();
                    }
                    UnityEngine.Object$$Destroy(pUVar74,(MethodInfo *)0x0);
                  }
                  SharedCreature$$RedrawLevelDisplay(pSVar26,(MethodInfo *)0x0);
                  return;
                }
              }
            }
          }
        }
      }
    }
    goto LAB_00958368;
  case 0x50:
    pSVar24 = Packet$$GetString(incoming,(MethodInfo *)0x0);
    pMVar103 = MobControl_TypeInfo->static_fields->Instance;
    if ((pMVar103 == (MobControl_o *)0x0) ||
       (pSVar55 = (pMVar103->fields).active_combatants,
       pSVar55 == (System_Collections_Generic_Dictionary_string__GameObject__o *)0x0))
    goto LAB_00958368;
    uVar32 = System.Collections.Generic.Dictionary<>$$ContainsKey
                       (pSVar55,pSVar24,
                        Method$System.Collections.Generic.Dictionary<string,-GameObject>.ContainsKey ()
                       );
    if ((uVar32 & 1) == 0) {
      return;
    }
    pMVar103 = MobControl_TypeInfo->static_fields->Instance;
    if ((pMVar103 == (MobControl_o *)0x0) ||
       (pSVar55 = (pMVar103->fields).active_combatants,
       pSVar55 == (System_Collections_Generic_Dictionary_string__GameObject__o *)0x0))
    goto LAB_00958368;
    pUVar74 = (UnityEngine_Object_o *)
              System.Collections.Generic.Dictionary<>$$get_Item
                        (pSVar55,pSVar24,
                         Method$System.Collections.Generic.Dictionary<string,-GameObject>.get_Item()
                        );
    if ((UnityEngine.Object_TypeInfo->_2).cctor_finished == 0) {
      thunk_FUN_008bc8d8(UnityEngine.Object_TypeInfo);
    }
    bVar5 = UnityEngine.Object$$op_Inequality(pUVar74,(UnityEngine_Object_o *)0x0,(MethodInfo *)0x0)
    ;
    if (!bVar5) {
      return;
    }
    if ((pUVar74 == (UnityEngine_Object_o *)0x0) ||
       (pIVar57 = UnityEngine.GameObject$$GetComponent<object>
                            ((UnityEngine_GameObject_o *)pUVar74,
                             Method$UnityEngine.GameObject.GetComponent<Combatant>()),
       pIVar57 == (Il2CppObject *)0x0)) goto LAB_00958368;
    if (*(int *)((long)&pIVar57[3].monitor + 4) != 3) {
      return;
    }
    pUVar56 = UnityEngine.GameObject$$get_gameObject
                        ((UnityEngine_GameObject_o *)pUVar74,(MethodInfo *)0x0);
    if ((pUVar56 == (UnityEngine_GameObject_o *)0x0) ||
       (pIVar57 = UnityEngine.GameObject$$GetComponent<object>
                            (pUVar56,Method$UnityEngine.GameObject.GetComponent<SharedCreature>()),
       pIVar57 == (Il2CppObject *)0x0)) goto LAB_00958368;
    if (*(char *)((long)&pIVar57[10].monitor + 4) != '\0') {
      return;
    }
    if (*(int *)&pIVar57[0xb].monitor != 6) {
      return;
    }
    pUVar74 = (UnityEngine_Object_o *)
              UnityEngine.GameObject$$get_gameObject
                        ((UnityEngine_GameObject_o *)pUVar74,(MethodInfo *)0x0);
    if ((UnityEngine.Object_TypeInfo->_2).cctor_finished == 0) {
      thunk_FUN_008bc8d8(UnityEngine.Object_TypeInfo);
    }
code_r0x00956fe0:
    UnityEngine.Object$$Destroy(pUVar74,(MethodInfo *)0x0);
    break;
  case 0x51:
    pSVar24 = Packet$$GetString(incoming,(MethodInfo *)0x0);
    iVar17 = Packet$$GetLong(incoming,(MethodInfo *)0x0);
    pSVar29 = Packet$$GetString(incoming,(MethodInfo *)0x0);
    pPVar28 = (PerkData_o *)thunk_FUN_008184f0(PerkData_TypeInfo);
    if (pPVar28 != (PerkData_o *)0x0) {
      PerkData$$.ctor(pPVar28,(MethodInfo *)0x0);
      PerkData$$UnpackFromWeb(pPVar28,incoming,(MethodInfo *)0x0);
      iVar10 = Packet$$GetShort(incoming,(MethodInfo *)0x0);
      pSVar63 = Packet$$GetString(incoming,(MethodInfo *)0x0);
      uVar6 = Packet$$GetByte(incoming,(MethodInfo *)0x0);
      pPVar64 = PlayerData_TypeInfo->static_fields->Instance;
      if (pPVar64 != (PlayerData_o *)0x0) {
        pSVar65 = PlayerData$$GetGlobalString(pPVar64,StringLiteral_14006,(MethodInfo *)0x0);
        bVar5 = System.String$$op_Equality(pSVar29,pSVar65,(MethodInfo *)0x0);
        pMVar103 = MobControl_TypeInfo->static_fields->Instance;
        pSVar65 = StringLiteral_5465;
        if (!bVar5) {
          pSVar65 = pSVar29;
        }
        if ((pMVar103 != (MobControl_o *)0x0) &&
           (pSVar55 = (pMVar103->fields).active_combatants,
           pSVar55 != (System_Collections_Generic_Dictionary_string__GameObject__o *)0x0)) {
          uVar32 = System.Collections.Generic.Dictionary<>$$ContainsKey
                             (pSVar55,pSVar65,
                              Method$System.Collections.Generic.Dictionary<string,-GameObject>.Conta insKey()
                             );
          if ((uVar32 & 1) == 0) {
            pUVar74 = (UnityEngine_Object_o *)0x0;
          }
          else {
            pMVar103 = MobControl_TypeInfo->static_fields->Instance;
            if ((pMVar103 == (MobControl_o *)0x0) ||
               (pSVar55 = (pMVar103->fields).active_combatants,
               pSVar55 == (System_Collections_Generic_Dictionary_string__GameObject__o *)0x0))
            goto LAB_00958368;
            pUVar74 = (UnityEngine_Object_o *)
                      System.Collections.Generic.Dictionary<>$$get_Item
                                (pSVar55,pSVar65,
                                 Method$System.Collections.Generic.Dictionary<string,-GameObject>.ge t_Item()
                                );
          }
          if ((UnityEngine.Object_TypeInfo->_2).cctor_finished == 0) {
            thunk_FUN_008bc8d8();
          }
          bVar5 = UnityEngine.Object$$op_Inequality
                            (pUVar74,(UnityEngine_Object_o *)0x0,(MethodInfo *)0x0);
          if (!bVar5) {
            return;
          }
          if ((pUVar74 != (UnityEngine_Object_o *)0x0) &&
             (pPVar66 = (PerkReceiver_o *)
                        UnityEngine.GameObject$$GetComponent<object>
                                  ((UnityEngine_GameObject_o *)pUVar74,
                                   Method$UnityEngine.GameObject.GetComponent<PerkReceiver>()),
             pPVar66 != (PerkReceiver_o *)0x0)) {
            PerkReceiver$$ApplyPerkEffect
                      (pPVar66,uVar6 == 1,pSVar63,pPVar28,(int)iVar10,pSVar24,iVar17,false,
                       (MethodInfo *)0x0);
            return;
          }
        }
      }
    }
    goto LAB_00958368;
  case 0x52:
    if ((UnityEngine.Debug_TypeInfo->_2).cctor_finished == 0) {
      thunk_FUN_008bc8d8();
    }
    UnityEngine.Debug$$Log(_StringLiteral_6902,(MethodInfo *)0x0);
    pPVar28 = (PerkData_o *)thunk_FUN_008184f0(PerkData_TypeInfo);
    if (pPVar28 != (PerkData_o *)0x0) {
      PerkData$$.ctor(pPVar28,(MethodInfo *)0x0);
      pMVar102 = (MethodInfo *)0x0;
      PerkData$$UnpackFromWeb(pPVar28,incoming,(MethodInfo *)0x0);
      iVar10 = Packet$$GetShort(incoming,(MethodInfo *)0x0);
      pSVar24 = Packet$$GetString(incoming,(MethodInfo *)0x0);
      pSVar29 = Packet$$GetString(incoming,(MethodInfo *)0x0);
      uVar18 = Packet$$GetLong(incoming,(MethodInfo *)0x0);
      UVar129 = GameServerReceiver$$UnpackPosition
                          ((GameServerReceiver_o *)(ulong)uVar18,incoming,pMVar102);
      UVar130 = GameServerReceiver$$UnpackPosition(__this_07,incoming,pMVar102);
      pPVar64 = PlayerData_TypeInfo->static_fields->Instance;
      if (pPVar64 != (PlayerData_o *)0x0) {
        pSVar63 = PlayerData$$GetGlobalString(pPVar64,StringLiteral_14006,(MethodInfo *)0x0);
        bVar5 = System.String$$op_Equality(pSVar24,pSVar63,(MethodInfo *)0x0);
        pPVar30 = PerkControl_TypeInfo->static_fields->Instance;
        if (pPVar30 != (PerkControl_o *)0x0) {
          pSVar63 = StringLiteral_5465;
          if (!bVar5) {
            pSVar63 = pSVar24;
          }
          PerkControl$$LaunchProjectile
                    (pPVar30,pPVar28,(int)iVar10,pSVar63,pSVar29,uVar18,UVar129,UVar130,
                     (MethodInfo *)0x0);
          return;
        }
      }
    }
    goto LAB_00958368;
  case 0x53:
    pSVar24 = Packet$$GetString(incoming,(MethodInfo *)0x0);
    uVar6 = Packet$$GetByte(incoming,(MethodInfo *)0x0);
    pMVar103 = MobControl_TypeInfo->static_fields->Instance;
    if ((pMVar103 == (MobControl_o *)0x0) ||
       (pSVar55 = (pMVar103->fields).active_combatants,
       pSVar55 == (System_Collections_Generic_Dictionary_string__GameObject__o *)0x0))
    goto LAB_00958368;
    uVar32 = System.Collections.Generic.Dictionary<>$$ContainsKey
                       (pSVar55,pSVar24,
                        Method$System.Collections.Generic.Dictionary<string,-GameObject>.ContainsKey ()
                       );
    if ((uVar32 & 1) != 0) {
      pMVar103 = MobControl_TypeInfo->static_fields->Instance;
      if (((pMVar103 == (MobControl_o *)0x0) ||
          (pSVar55 = (pMVar103->fields).active_combatants,
          pSVar55 == (System_Collections_Generic_Dictionary_string__GameObject__o *)0x0)) ||
         ((pUVar56 = (UnityEngine_GameObject_o *)
                     System.Collections.Generic.Dictionary<>$$get_Item
                               (pSVar55,pSVar24,
                                Method$System.Collections.Generic.Dictionary<string,-GameObject>.get _Item()
                               ), pUVar56 == (UnityEngine_GameObject_o *)0x0 ||
          (pIVar57 = UnityEngine.GameObject$$GetComponent<object>
                               (pUVar56,Method$UnityEngine.GameObject.GetComponent<SharedCreature>()
                               ), pIVar57 == (Il2CppObject *)0x0)))) goto LAB_00958368;
      *(bool *)((long)&pIVar57[5].klass + 2) = uVar6 == 1;
    }
    break;
  case 0x54:
    pSVar24 = Packet$$GetString(incoming,(MethodInfo *)0x0);
    iVar10 = Packet$$GetShort(incoming,(MethodInfo *)0x0);
    iVar19 = (int)iVar10;
    if (0 < iVar19) {
      do {
        pSVar29 = Packet$$GetString(incoming,(MethodInfo *)0x0);
        iVar17 = Packet$$GetLong(incoming,(MethodInfo *)0x0);
        pPVar28 = (PerkData_o *)thunk_FUN_008184f0(PerkData_TypeInfo);
        if (pPVar28 == (PerkData_o *)0x0) goto LAB_00958368;
        PerkData$$.ctor(pPVar28,(MethodInfo *)0x0);
        PerkData$$UnpackFromWeb(pPVar28,incoming,(MethodInfo *)0x0);
        iVar10 = Packet$$GetShort(incoming,(MethodInfo *)0x0);
        pSVar63 = Packet$$GetString(incoming,(MethodInfo *)0x0);
        iVar11 = Packet$$GetShort(incoming,(MethodInfo *)0x0);
        Packet$$GetShort(incoming,(MethodInfo *)0x0);
        pPVar64 = PlayerData_TypeInfo->static_fields->Instance;
        if (pPVar64 == (PlayerData_o *)0x0) goto LAB_00958368;
        pSVar65 = PlayerData$$GetGlobalString(pPVar64,StringLiteral_14006,(MethodInfo *)0x0);
        bVar5 = System.String$$op_Equality(pSVar29,pSVar65,(MethodInfo *)0x0);
        pMVar103 = MobControl_TypeInfo->static_fields->Instance;
        pSVar65 = StringLiteral_5465;
        if (!bVar5) {
          pSVar65 = pSVar29;
        }
        if ((pMVar103 == (MobControl_o *)0x0) ||
           (pSVar55 = (pMVar103->fields).active_combatants,
           pSVar55 == (System_Collections_Generic_Dictionary_string__GameObject__o *)0x0))
        goto LAB_00958368;
        uVar32 = System.Collections.Generic.Dictionary<>$$ContainsKey
                           (pSVar55,pSVar24,
                            Method$System.Collections.Generic.Dictionary<string,-GameObject>.Contain sKey()
                           );
        if ((uVar32 & 1) != 0) {
          pMVar103 = MobControl_TypeInfo->static_fields->Instance;
          if ((((pMVar103 == (MobControl_o *)0x0) ||
               (pSVar55 = (pMVar103->fields).active_combatants,
               pSVar55 == (System_Collections_Generic_Dictionary_string__GameObject__o *)0x0)) ||
              (pUVar56 = (UnityEngine_GameObject_o *)
                         System.Collections.Generic.Dictionary<>$$get_Item
                                   (pSVar55,pSVar24,
                                    Method$System.Collections.Generic.Dictionary<string,-GameObject> .get_Item()
                                   ), pUVar56 == (UnityEngine_GameObject_o *)0x0)) ||
             (pPVar66 = (PerkReceiver_o *)
                        UnityEngine.GameObject$$GetComponent<object>
                                  (pUVar56,
                                   Method$UnityEngine.GameObject.GetComponent<PerkReceiver>()),
             pPVar66 == (PerkReceiver_o *)0x0)) goto LAB_00958368;
          PerkReceiver$$InitializeDurationTimer
                    (pPVar66,(int)iVar11,pSVar63,pPVar28,(int)iVar10,pSVar65,iVar17,true,
                     (MethodInfo *)0x0);
        }
        iVar19 = iVar19 + -1;
      } while (iVar19 != 0);
    }
    break;
  case 0x55:
    UVar129 = GameServerReceiver$$UnpackPosition
                        ((GameServerReceiver_o *)
                         &Method$UnityEngine.GameObject.GetComponent<Combatant>(),incoming,
                         (MethodInfo *)&Startup_TypeInfo);
    pSVar24 = Packet$$GetString(incoming,(MethodInfo *)0x0);
    pPVar28 = (PerkData_o *)thunk_FUN_008184f0(PerkData_TypeInfo);
    if (pPVar28 != (PerkData_o *)0x0) {
      PerkData$$.ctor(pPVar28,(MethodInfo *)0x0);
      PerkData$$UnpackFromWeb(pPVar28,incoming,(MethodInfo *)0x0);
      iVar10 = Packet$$GetShort(incoming,(MethodInfo *)0x0);
      pSVar29 = Packet$$GetString(incoming,(MethodInfo *)0x0);
      iVar17 = Packet$$GetLong(incoming,(MethodInfo *)0x0);
      if ((UnityEngine.Debug_TypeInfo->_2).cctor_finished == 0) {
        thunk_FUN_008bc8d8(UnityEngine.Debug_TypeInfo);
      }
      UnityEngine.Debug$$Log(_StringLiteral_4499,(MethodInfo *)0x0);
      pPVar30 = PerkControl_TypeInfo->static_fields->Instance;
      if (pPVar30 != (PerkControl_o *)0x0) {
        PerkControl$$CreateDrop
                  (pPVar30,UVar129,pSVar24,pPVar28,(int)iVar10,pSVar29,iVar17,false,
                   (MethodInfo *)0x0);
        return;
      }
    }
    goto LAB_00958368;
  case 0x56:
    pSVar24 = Packet$$GetString(incoming,(MethodInfo *)0x0);
    __this_02 = (OnlinePlayerData_o *)thunk_FUN_008184f0(OnlinePlayerData_TypeInfo);
    if (__this_02 != (OnlinePlayerData_o *)0x0) {
      OnlinePlayerData$$.ctor(__this_02,(MethodInfo *)0x0);
      OnlinePlayerData$$Unpack(__this_02,incoming,(MethodInfo *)0x0);
      pGVar95 = GameServerInterface_TypeInfo->static_fields->Instance;
      if ((pGVar95 != (GameServerInterface_o *)0x0) &&
         (pSVar39 = (pGVar95->fields).nearby_players,
         pSVar39 != (System_Collections_Generic_Dictionary_string__OnlinePlayer__o *)0x0)) {
        uVar32 = System.Collections.Generic.Dictionary<>$$ContainsKey
                           (pSVar39,pSVar24,
                            Method$System.Collections.Generic.Dictionary<string,-OnlinePlayer>.Conta insKey()
                           );
        if ((uVar32 & 1) == 0) {
          return;
        }
        pMVar103 = MobControl_TypeInfo->static_fields->Instance;
        if ((pMVar103 != (MobControl_o *)0x0) &&
           (pSVar55 = (pMVar103->fields).active_combatants,
           pSVar55 != (System_Collections_Generic_Dictionary_string__GameObject__o *)0x0)) {
          uVar32 = System.Collections.Generic.Dictionary<>$$ContainsKey
                             (pSVar55,pSVar24,
                              Method$System.Collections.Generic.Dictionary<string,-GameObject>.Conta insKey()
                             );
          if ((uVar32 & 1) != 0) {
            return;
          }
          pGVar95 = GameServerInterface_TypeInfo->static_fields->Instance;
          if ((pGVar95 != (GameServerInterface_o *)0x0) &&
             (pSVar39 = (pGVar95->fields).nearby_players,
             pSVar39 != (System_Collections_Generic_Dictionary_string__OnlinePlayer__o *)0x0)) {
            pMVar103 = MobControl_TypeInfo->static_fields->Instance;
            player = (OnlinePlayer_o *)
                     System.Collections.Generic.Dictionary<>$$get_Item
                               (pSVar39,pSVar24,
                                Method$System.Collections.Generic.Dictionary<string,-OnlinePlayer>.g et_Item()
                               );
            if (pMVar103 != (MobControl_o *)0x0) {
              MobControl$$SpawnOtherPlayer(pMVar103,player,__this_02,(MethodInfo *)0x0);
              return;
            }
          }
        }
      }
    }
    goto LAB_00958368;
  case 0x58:
    pSVar24 = Packet$$GetString(incoming,(MethodInfo *)0x0);
    uVar6 = Packet$$GetByte(incoming,(MethodInfo *)0x0);
    uVar18 = (uint)uVar6;
    _Var53.rgctx_data =
         (Il2CppRGCTXData *)thunk_FUN_008184f0(System.Collections.Generic.List<string>_TypeInfo);
    if (_Var53.rgctx_data == (Il2CppRGCTXData *)0x0) goto LAB_00958368;
    System.Collections.Generic.List<object>$$.ctor
              ((System_Collections_Generic_List_object__o *)_Var53.rgctx_data,
               Method$System.Collections.Generic.List<string>..ctor());
    if (uVar6 != 0) {
      do {
        pIVar54 = (Il2CppClass *)Packet$$GetString(incoming,(MethodInfo *)0x0);
        lVar61 = Method$System.Collections.Generic.List<string>.Add();
        pMVar102 = _Var53.rgctx_data[2].method;
        *(int *)((long)_Var53.rgctx_data + 0x1c) = *(int *)((long)_Var53.rgctx_data + 0x1c) + 1;
        if (pMVar102 == (MethodInfo *)0x0) goto LAB_00958368;
        uVar2 = *(uint *)(_Var53.rgctx_data + 3);
        if (uVar2 < *(uint *)&pMVar102->name) {
          *(uint *)(_Var53.rgctx_data + 3) = uVar2 + 1;
          (&pMVar102->klass)[(int)uVar2] = pIVar54;
          thunk_FUN_008c6b48();
        }
        else {
          (**(code **)(*(long *)(*(long *)(*(long *)(lVar61 + 0x20) + 0xc0) + 0x58) + 8))
                    (_Var53.rgctx_data);
        }
        uVar18 = uVar18 - 1;
      } while (uVar18 != 0);
    }
    pMVar103 = MobControl_TypeInfo->static_fields->Instance;
    if ((pMVar103 == (MobControl_o *)0x0) ||
       (pSVar55 = (pMVar103->fields).active_combatants,
       pSVar55 == (System_Collections_Generic_Dictionary_string__GameObject__o *)0x0))
    goto LAB_00958368;
    uVar32 = System.Collections.Generic.Dictionary<>$$ContainsKey
                       (pSVar55,pSVar24,
                        Method$System.Collections.Generic.Dictionary<string,-GameObject>.ContainsKey ()
                       );
    if ((uVar32 & 1) == 0) {
      return;
    }
    pMVar103 = MobControl_TypeInfo->static_fields->Instance;
    if (((pMVar103 == (MobControl_o *)0x0) ||
        (pSVar55 = (pMVar103->fields).active_combatants,
        pSVar55 == (System_Collections_Generic_Dictionary_string__GameObject__o *)0x0)) ||
       ((pUVar56 = (UnityEngine_GameObject_o *)
                   System.Collections.Generic.Dictionary<>$$get_Item
                             (pSVar55,pSVar24,
                              Method$System.Collections.Generic.Dictionary<string,-GameObject>.get_I tem()
                             ), pUVar56 == (UnityEngine_GameObject_o *)0x0 ||
        (pIVar57 = UnityEngine.GameObject$$GetComponent<object>
                             (pUVar56,Method$UnityEngine.GameObject.GetComponent<SharedCreature>()),
        pIVar57 == (Il2CppObject *)0x0)))) goto LAB_00958368;
    p_Var73 = (_union_13 *)(pIVar57 + 0xb);
    p_Var73->rgctx_data = (Il2CppRGCTXData *)_Var53;
code_r0x00954f68:
    thunk_FUN_008c6b48(p_Var73,_Var53.rgctx_data);
    break;
  case 0x5a:
    pSVar24 = Packet$$GetString(incoming,(MethodInfo *)0x0);
    pBVar94 = BanditCampsControl_TypeInfo->static_fields->Instance;
    if ((pBVar94 != (BanditCampsControl_o *)0x0) &&
       (pSVar31 = (pBVar94->fields).loaded_bandit_camp_instances,
       pSVar31 != (System_Collections_Generic_Dictionary_string__BanditCampInstance__o *)0x0)) {
      uVar32 = System.Collections.Generic.Dictionary<>$$ContainsKey
                         (pSVar31,pSVar24,
                          Method$System.Collections.Generic.Dictionary<string,-BanditCampInstance>.C ontainsKey()
                         );
      if ((uVar32 & 1) == 0) {
        return;
      }
      pBVar94 = BanditCampsControl_TypeInfo->static_fields->Instance;
      if (((pBVar94 != (BanditCampsControl_o *)0x0) &&
          (pSVar31 = (pBVar94->fields).loaded_bandit_camp_instances,
          pSVar31 != (System_Collections_Generic_Dictionary_string__BanditCampInstance__o *)0x0)) &&
         (pBVar33 = (BanditCampInstance_o *)
                    System.Collections.Generic.Dictionary<>$$get_Item
                              (pSVar31,pSVar24,
                               Method$System.Collections.Generic.Dictionary<string,-BanditCampInstan ce>.get_Item()
                              ), pBVar33 != (BanditCampInstance_o *)0x0)) {
        (pBVar33->fields).flag_destroyed = true;
        pGVar90 = GameServerConnector_TypeInfo->static_fields->Instance;
        if (pGVar90 != (GameServerConnector_o *)0x0) {
          if ((pGVar90->fields).is_host_cached == false) {
            return;
          }
          BanditCampInstance$$SaveToDisk(pBVar33,(MethodInfo *)0x0);
          return;
        }
      }
    }
LAB_00958368:
                    /* WARNING: Subroutine does not return */
    FUN_0083c89c();
  }
LAB_0094fa28:
  return;
}

