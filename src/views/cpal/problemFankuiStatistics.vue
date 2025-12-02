<template>
  <div class="deliveryProgress">
    <div class="searchLeft">
      <!--<p>车位管理</p>-->
      <div>
        <checked-tree @getPostId="searchProcess" @getPostDeptId="searchDeptProcess"></checked-tree>
      </div>
    </div>
    <div class="detailRight">
      <div class="topBox">
        <span>年度：</span>
        <el-date-picker v-model="data" type="year" placeholder="选择年" value-format="yyyy" @change="datePickerChange">
        </el-date-picker>
        <el-button type="primary" class="buttonCs" @click="query">查询</el-button>
        <el-button class="buttonCs" @click="reset">重置</el-button>
      </div>
      <div class="collapse-container" v-show="deptFlag === '1'">
        <div class="collapse_Bigbox">
          <el-collapse v-model="activeNames" v-for="(item, index) in combinedDataList" :key="index"
            @change="handleChange" class="collapse_class">
            <el-collapse-item :name="index">
              <template slot="title">
                <div class="el-collapse-item__headers">
                  <div class="headerBuilding" style="margin-left: 20px; font-weight: 600;
      font-size: 16px;">
                    <img src="@/assets/images/problemZong/building.png" alt="" style="margin-right: 10px;">
                    <span>楼盘:{{ item.name || '——' }}</span>
                  </div>
                  <div class="headerBottom">
                    <div class="headerBottom_items" style="margin-right: 10px;background-color: #EFFAF3;">
                      <div class="headerBottom_item">
                        <img src="@/assets/images/problemZong/k.png" alt="">
                        <span style="margin-left: 5px;">排名</span>
                      </div>
                      <span>{{ item.paiming || '-' }}</span>
                    </div>
                    <div class="headerBottom_items" style="background-color: #E5F2FF;">
                      <div class="headerBottom_item">
                        <img src="@/assets/images/problemZong/L.png" alt="">
                        <span style="margin-left: 5px;">评分</span>
                      </div>
                      <span>{{ item.zongCount || '-' }}</span>
                    </div>
                  </div>

                </div>
              </template>
              <div class="bottomBox">
                <div class="detailRightTop">
                  <div class="detailRightTopTitle">
                    <i class="el-icon-s-data" style="color: #409eff;"></i>
                    <p>问题类型统计</p>
                  </div>
                  <!--height: calc(100% - 30px)-->
                  <div :id="'detailRightTopEcharts' + index" style="width:100%;height:  calc(100% - 30px);"
                    v-show="item.hasData" ref="chartContainerProType">
                  </div>
                  <div class="detailRightTopEcharts noData" v-show="!item.hasData">暂无数据</div>
                </div>
                <div class="detailRightbottom">
                  <div class="detailRightbottomLeft">
                    <div class="detailRightTopTitle">
                      <i class="el-icon-s-data" style="color: #409eff;"></i>
                      <p>
                        问题处理状态
                      </p>
                    </div>
                    <div class="detailRightbottomLeftEcharts" id="detailRightbottomLeftEcharts"
                      style="width: 100%;height:calc(100% - 30px);" v-show="!getProblemStatsNoData"></div>
                    <div class="detailRightbottomLeftEcharts noData" v-show="getProblemStatsNoData">暂无数据</div>
                  </div>
                  <div class="detailRightbottomRight">
                    <div class="detailRightTopTitle">
                      <i class="el-icon-s-data" style="color: #409eff;"></i>
                      <p>处置方统计</p>
                    </div>
                    <div class="detailRightbottomRightEcharts" id="detailRightbottomRightEcharts"
                      style="width: 100%;height:calc(100% - 30px);" v-show="!getProblemStatsNoData"></div>
                    <div class="detailRightbottomRightEcharts noData" v-show="getProblemStatsNoData">暂无数据</div>

                  </div>
                </div>
                <div class="content_class" style="margin-top: 10px;margin-bottom: 10px;">
                  <div class="card">
                    <div class="cardContentTwo">
                      <div class="cardContentTwoItem">
                        <div class="cardContentTwoItemTextTitle">
                          <i class="el-icon-s-data" style="color: #409eff;"></i>
                          <p>交房满意度</p>
                        </div>
                        <div class="cardContentTwoItemNumberItem">
                          <div class="cardContentTwoItemNumber">
                            <span>总问卷数量</span>
                            <span style="margin-top: 10px;">{{ item.jiaofuSum || '-' }}份</span>
                          </div>
                          <div class="cardContentTwoItemNumber">
                            <span>
                              总平均分
                            </span>
                            <span style="margin-top: 10px;">
                              {{ item.averageScore || '-' }}分
                            </span>
                          </div>
                          <div class="cardContentTwoItemNumber">
                            <span>
                              有效问卷数
                            </span>
                            <span style="margin-top: 10px;">
                              {{ item.wuxiaoWenjuansum || '-' }}份
                            </span>
                          </div>
                          <div class="cardContentTwoItemNumber">
                            <span>
                              有效平均分
                            </span>
                            <span style="margin-top: 10px;">
                              {{ item.invalidAverageScore || '-' }}分
                            </span>
                          </div>
                        </div>
                      </div>
                      <div class="cardContentTwoItem">
                        <span class="cardContentTwoItemTextTitle">
                          <i class="el-icon-s-data" style="color: #409eff;"></i>
                          <p>入住体验反馈</p>
                        </span>
                        <div class="cardContentTwoItemNumberItem">
                          <div class="cardContentTwoItemNumber">
                            <span>总问卷数量</span>
                            <span style="margin-top: 10px;">
                              {{ item.problemCountRZ || '-' }}份
                            </span>
                          </div>
                          <div class="cardContentTwoItemNumber">
                            <span>
                              总平均分
                            </span>
                            <span style="margin-top: 10px;">
                              {{ item.averageScoreRZ || '-' }}分
                            </span>
                          </div>
                          <div class="cardContentTwoItemNumber">
                            <span>
                              有效问卷数
                            </span>
                            <span style="margin-top: 10px;">
                              {{ item.invalidProblemCountRZ || '-' }}份
                            </span>
                          </div>
                          <div class="cardContentTwoItemNumber">
                            <span>
                              有效平均分
                            </span>
                            <span style="margin-top: 10px;">
                              {{ item.invalidAverageScoreRZ || '-' }}分
                            </span>
                          </div>
                        </div>
                      </div>
                    </div>
                    <div class="problemSummaryTitle">
                      <div style="background-color: #fff;padding-top: 10px;border-radius: 5px;">
                        <div style="font-size: 14px;
                          font-weight: bold;
                          height: 20px;
                          margin-bottom: 10px;
                          display: flex;
                          align-items: center;
                          padding-left: 22px;">
                          <i class="el-icon-s-data" style="color: #409eff;"></i>
                          <p style="margin-left: 5px;">问题汇总</p>
                        </div>
                        <template>
                          <div class="problemTj" style="flex: 1;">
                            <div class="problemTjItem" style="background-color: #e4f3ff;">
                              <div class="problemTjItem_item">
                                <img src="@/assets/images/problemZong/a.png" alt="">
                                <span style="margin-left: 5px;">问题总数量(个)</span>
                              </div>
                              <span>{{ item.proCount || '-' }}</span>
                            </div>
                            <div class="problemTjItem" style="background-color: #faefeb;">
                              <div class="problemTjItem_item">
                                <img src="@/assets/images/problemZong/b.png" alt="">
                                <span style="margin-left: 5px;">已完成(个)</span>
                              </div>
                              <span>{{ item.doneProCount || '-' }}</span>
                            </div>
                            <div class="problemTjItem" style="background-color: #e9ecfa;">
                              <div class="problemTjItem_item">
                                <img src="@/assets/images/problemZong/c.png" alt="">
                                <span style="margin-left: 5px;">未完成(个)</span>
                              </div>
                              <span>{{ item.unDoneProCount || '-' }}</span>
                            </div>
                            <div class="problemTjItem" style="background-color: #fdf3e5;">
                              <div class="problemTjItem_item">
                                <img src="@/assets/images/problemZong/d.png" alt="">
                                <span style="margin-left: 5px;">完成率(%)</span>
                              </div>
                              <span>{{ item.proRate || '-' }}</span>
                            </div>
                          </div>
                        </template>
                        <div class="content">
                          <div class="problemCZFTJ" style="margin-top: 20px;">
                            <div style="font-size: 14px;
                          font-weight: bold;
                          height: 20px;
                          margin-bottom: 10px;
                          display: flex;
                          align-items: center;
                          padding-left: 22px;">
                              <i class="el-icon-s-data" style="color: #409eff;"></i>
                              <p style="margin-left: 5px;">处置方统计</p>
                            </div>
                            <div class="problemTj_item">
                              <div class="problemTjZong">
                                <div class="problemTjItem">

                                  <img src="@/assets/images/problemZong/e.png" alt="">
                                  <div class="problemTjItem_items">
                                    <span>工程问题总数量(个)</span>
                                    <span>{{ item.gclProCount || '-' }}</span>
                                  </div>

                                </div>
                                <div class="problemTjItem">

                                  <img src="@/assets/images/problemZong/g.png" alt="">
                                  <div class="problemTjItem_items">
                                    <span>已完成(个)</span>
                                    <span>{{ item.gclProDoneCount || '-' }}</span>
                                  </div>

                                </div>
                                <div class="problemTjItem">
                                  <img src="@/assets/images/problemZong/f.png" alt="">
                                  <div class="problemTjItem_items">
                                    <span>未完成(个)</span>
                                    <span>{{ item.gclProUnDoneCount || '-' }}</span>
                                  </div>
                                </div>
                                <div class="problemTjItem">
                                  <img src="@/assets/images/problemZong/h.png" alt="">
                                  <div class="problemTjItem_items">
                                    <span>完成率(%)</span>
                                    <span>{{ item.gclProRate || '-' }}</span>
                                  </div>

                                </div>
                              </div>
                              <div class="problemTjZong"
                                style="width: 1px;height: 180px;background-color: rgb(228, 228, 228,.8);">
                              </div>
                              <div class="problemTjZong">
                                <div class="problemTjItem">
                                  <img src="@/assets/images/problemZong/e.png" alt="">
                                  <div class="problemTjItem_items">
                                    <span>物业问题总数量(个)</span>
                                    <span>{{ item.wyProCount || '-' }}</span>
                                  </div>

                                </div>
                                <div class="problemTjItem">
                                  <img src="@/assets/images/problemZong/g.png" alt="">

                                  <div class="problemTjItem_items">
                                    <span>已完成(个)</span>
                                    <span>{{ item.wyProDoneCount || '-' }}</span>
                                  </div>

                                </div>
                                <div class="problemTjItem">
                                  <img src="@/assets/images/problemZong/f.png" alt="">

                                  <div class="problemTjItem_items">
                                    <span>未完成(个)</span>
                                    <span>{{ item.wyProUnDoneCount || '-' }}</span>
                                  </div>

                                </div>
                                <div class="problemTjItem">
                                  <img src="@/assets/images/problemZong/h.png" alt=""></img>

                                  <div class="problemTjItem_items">
                                    <span>完成率(%)</span>
                                    <span>{{ item.wyProRate || '-' }}</span>
                                  </div>

                                </div>
                              </div>
                            </div>

                          </div>
                        </div>
                      </div>
                      <div class="problemTypeTj"
                        style="margin-top: 10px;padding-top:10px;background-color:#fff;border-radius: 5px;margin-bottom: 20px;">
                        <div style="font-size: 14px;
                          font-weight: bold;
                          height: 20px;
                          margin-bottom: 10px;
                          display: flex;
                          align-items: center;
                          padding-left: 22px;">
                          <i class="el-icon-s-data" style="color: #409eff;"></i>
                          <p style="margin-left: 5px;">问题类型统计</p>
                        </div>
                        <div class="problemTjTable">
                          <div class="uni-container">
                            <el-table :data="item.problemTypeTableData" style="width: 100%" :stripe="true"
                              :header-cell-style="{
                                background: '#ecf4fa'
                              }" :row-class-name="onTableRowClassName">
                              <el-table-column prop="questionName" label="问题类型" align="center">
                              </el-table-column>
                              <el-table-column prop="proTypeDoneCount" label="已完成(个)" align="center">
                              </el-table-column>
                              <el-table-column prop="proTypeUnDoneCount" label="未完成(个)" align="center">
                              </el-table-column>
                              <el-table-column prop="proTypeRate" label="完成率(%)" align="center">
                              </el-table-column>
                            </el-table>
                          </div>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </el-collapse-item>
          </el-collapse>
        </div>
      </div>

      <div class="bottomBox bottomBoxTwo" v-show="deptFlag === '0'" v-for="(item, index) in combinedDataList"
        :key="index">
        <div class="headerBottom">
          <div class="headerBottom_items" style="margin-right: 10px;background-color: #EFFAF3;">
            <div class="headerBottom_item">
              <img src="@/assets/images/problemZong/k.png" alt="">
              <span style="margin-left: 5px;">排名</span>
            </div>
            <span>{{ item.paiming || '-' }}</span>
          </div>
          <div class="headerBottom_items" style="background-color: #E5F2FF;">
            <div class="headerBottom_item">
              <img src="@/assets/images/problemZong/L.png" alt="">
              <span style="margin-left: 5px;">评分</span>
            </div>
            <span>{{ item.zongCount || '-' }}</span>
          </div>
          <div class="headerBottom_items" style="background-color: #E5F2FF;">
            <div class="headerBottom_item">
              <img src="@/assets/images/problemZong/L.png" alt="">
              <span style="margin-left: 5px;">有效评分</span>
            </div>
            <span>{{ item.zongYouxiaoCount || '-' }}</span>
          </div>
        </div>
        <div class="detailRightTop">
          <div class="detailRightTopTitle">
            <i class="el-icon-s-data" style="color: #409eff;"></i>
            <p>问题类型统计</p>
          </div>
          <div id="detailRightTopEchartsTwo" style="width:100%;height: calc(100% - 30px);"
            v-show="!getProblemByTypeNoData">
          </div>
          <div class="detailRightTopEcharts noData" v-show="getProblemByTypeNoData">暂无数据</div>
        </div>
        <div class="detailRightbottom">
          <div class="detailRightbottomLeft">
            <div class="detailRightTopTitle">
              <i class="el-icon-s-data" style="color: #409eff;"></i>
              <p>
                问题处理状态
              </p>
            </div>
            <div class="detailRightbottomLeftEcharts" id="detailRightbottomLeftEchartsTwo"
              style="width: 100%;height: calc(100% - 30px);" v-show="!getProblemStatsNoData"></div>
            <div class="detailRightbottomLeftEcharts noData" v-show="getProblemStatsNoData">暂无数据</div>
          </div>
          <div class="detailRightbottomRight">
            <div class="detailRightTopTitle">
              <i class="el-icon-s-data" style="color: #409eff;"></i>
              <p>处置方统计</p>
            </div>
            <div class="detailRightbottomRightEcharts" id="detailRightbottomRightEchartsTwo"
              style="width: 100%;height: calc(100% - 30px);" v-show="!getProblemStatsNoData"></div>
            <div class="detailRightbottomRightEcharts noData" v-show="getProblemStatsNoData">暂无数据</div>

          </div>
        </div>
        <div class="content_class" style="margin-top: 10px;margin-bottom: 10px;">
          <div class="card" v-for="(item, index) in combinedDataList" :key="index">
            <div class="cardContentTwo">
              <div class="cardContentTwoItem">
                <div class="cardContentTwoItemTextTitle">
                  <i class="el-icon-s-data" style="color: #409eff;"></i>
                  <p>交房满意度</p>
                </div>
                <div class="cardContentTwoItemNumberItem">
                  <div class="cardContentTwoItemNumber">
                    <span>问卷数量</span>
                    <span style="margin-top: 10px;">{{ item.problemCount || '-' }}份</span>
                  </div>
                  <div class="cardContentTwoItemNumber">
                    <span>
                      平均分
                    </span>
                    <span style="margin-top: 10px;">
                      {{ item.averageScore || '-' }}分
                    </span>
                  </div>
                  <div class="cardContentTwoItemNumber">
                    <span>
                      无效问卷数
                    </span>
                    <span style="margin-top: 10px;">
                      {{ item.invalidProblemCount || '-' }}份
                    </span>
                  </div>
                  <div class="cardContentTwoItemNumber">
                    <span>
                      无效平均分
                    </span>
                    <span style="margin-top: 10px;">
                      {{ item.invalidAverageScore || '-' }}分
                    </span>
                  </div>
                </div>
              </div>
              <div class="cardContentTwoItem">
                <span class="cardContentTwoItemTextTitle">
                  <i class="el-icon-s-data" style="color: #409eff;"></i>
                  <p>入住体验反馈</p>
                </span>
                <div class="cardContentTwoItemNumberItem">
                  <div class="cardContentTwoItemNumber">
                    <span>问卷数量</span>
                    <span style="margin-top: 10px;">
                      {{ item.problemCountRZ || '-' }}份
                    </span>
                  </div>
                  <div class="cardContentTwoItemNumber">
                    <span>
                      平均分
                    </span>
                    <span style="margin-top: 10px;">
                      {{ item.averageScoreRZ || '-' }}分
                    </span>
                  </div>
                  <div class="cardContentTwoItemNumber">
                    <span>
                      无效问卷数
                    </span>
                    <span style="margin-top: 10px;">
                      {{ item.invalidProblemCountRZ || '-' }}份
                    </span>
                  </div>
                  <div class="cardContentTwoItemNumber">
                    <span>
                      无效平均分
                    </span>
                    <span style="margin-top: 10px;">
                      {{ item.invalidAverageScoreRZ || '-' }}分
                    </span>
                  </div>
                </div>
              </div>
            </div>
            <div class="problemSummaryTitle">
              <div style="background-color: #fff;padding-top: 10px;border-radius: 5px;">
                <div style="font-size: 14px;
                          font-weight: bold;
                          height: 20px;
                          margin-bottom: 10px;
                          display: flex;
                          align-items: center;
                          padding-left: 22px;">
                  <i class="el-icon-s-data" style="color: #409eff;"></i>
                  <p style="margin-left: 5px;">问题汇总</p>
                </div>
                <template>
                  <div class="problemTj" style="flex: 1;">
                    <div class="problemTjItem" style="background-color: #e4f3ff;">
                      <div class="problemTjItem_item">
                        <img src="@/assets/images/problemZong/a.png" alt="">
                        <span style="margin-left: 5px;">问题总数量(个)</span>
                      </div>
                      <span>{{ item.proCount || '-' }}</span>
                    </div>
                    <div class="problemTjItem" style="background-color: #faefeb;">
                      <div class="problemTjItem_item">
                        <img src="@/assets/images/problemZong/b.png" alt="">
                        <span style="margin-left: 5px;">已完成(个)</span>
                      </div>
                      <span>{{ item.doneProCount || '-' }}</span>
                    </div>
                    <div class="problemTjItem" style="background-color: #e9ecfa;">
                      <div class="problemTjItem_item">
                        <img src="@/assets/images/problemZong/c.png" alt="">
                        <span style="margin-left: 5px;">未完成(个)</span>
                      </div>
                      <span>{{ item.unDoneProCount || '-' }}</span>
                    </div>
                    <div class="problemTjItem" style="background-color: #fdf3e5;">
                      <div class="problemTjItem_item">
                        <img src="@/assets/images/problemZong/d.png" alt="">
                        <span style="margin-left: 5px;">完成率(%)</span>
                      </div>
                      <span>{{ item.proRate || '-' }}</span>
                    </div>
                  </div>
                </template>
                <div class="content">
                  <div class="problemCZFTJ" style="margin-top: 20px;">
                    <div style="font-size: 14px;
                          font-weight: bold;
                          height: 20px;
                          margin-bottom: 10px;
                          display: flex;
                          align-items: center;
                          padding-left: 22px;">
                      <i class="el-icon-s-data" style="color: #409eff;"></i>
                      <p style="margin-left: 5px;">处置方统计</p>
                    </div>
                    <div class="problemTj_item">
                      <div class="problemTjZong">
                        <div class="problemTjItem">

                          <img src="@/assets/images/problemZong/e.png" alt="">
                          <div class="problemTjItem_items">
                            <span>工程问题总数量(个)</span>
                            <span>{{ item.gclProCount || '-' }}</span>
                          </div>

                        </div>
                        <div class="problemTjItem">

                          <img src="@/assets/images/problemZong/g.png" alt="">
                          <div class="problemTjItem_items">
                            <span>已完成(个)</span>
                            <span>{{ item.gclProDoneCount || '-' }}</span>
                          </div>

                        </div>
                        <div class="problemTjItem">
                          <img src="@/assets/images/problemZong/f.png" alt="">
                          <div class="problemTjItem_items">
                            <span>未完成(个)</span>
                            <span>{{ item.gclProUnDoneCount || '-' }}</span>
                          </div>
                        </div>
                        <div class="problemTjItem">
                          <img src="@/assets/images/problemZong/h.png" alt="">
                          <div class="problemTjItem_items">
                            <span>完成率(%)</span>
                            <span>{{ item.gclProRate || '-' }}</span>
                          </div>

                        </div>
                      </div>
                      <div class="problemTjZong"
                        style="width: 1px;height: 180px;background-color: rgb(228, 228, 228,.8);">
                      </div>
                      <div class="problemTjZong">
                        <div class="problemTjItem">
                          <img src="@/assets/images/problemZong/e.png" alt="">
                          <div class="problemTjItem_items">
                            <span>物业问题总数量(个)</span>
                            <span>{{ item.wyProCount || '-' }}</span>
                          </div>

                        </div>
                        <div class="problemTjItem">
                          <img src="@/assets/images/problemZong/g.png" alt="">

                          <div class="problemTjItem_items">
                            <span>已完成(个)</span>
                            <span>{{ item.wyProDoneCount || '-' }}</span>
                          </div>

                        </div>
                        <div class="problemTjItem">
                          <img src="@/assets/images/problemZong/f.png" alt="">

                          <div class="problemTjItem_items">
                            <span>未完成(个)</span>
                            <span>{{ item.wyProUnDoneCount || '-' }}</span>
                          </div>

                        </div>
                        <div class="problemTjItem">
                          <img src="@/assets/images/problemZong/h.png" alt=""></img>

                          <div class="problemTjItem_items">
                            <span>完成率(%)</span>
                            <span>{{ item.wyProRate || '-' }}</span>
                          </div>

                        </div>
                      </div>
                    </div>

                  </div>
                </div>
              </div>
              <div class="problemTypeTj"
                style="margin-top: 10px;padding-top:10px;background-color:#fff;border-radius: 5px;margin-bottom: 20px;">
                <div style="font-size: 14px;
                          font-weight: bold;
                          height: 20px;
                          margin-bottom: 10px;
                          display: flex;
                          align-items: center;
                          padding-left: 22px;">
                  <i class="el-icon-s-data" style="color: #409eff;"></i>
                  <p style="margin-left: 5px;">问题类型统计</p>
                </div>
                <div class="problemTjTable">
                  <div class="uni-container">
                    <el-table :data="item.problemTypeTableData" style="width: 100%" :stripe="true" :header-cell-style="{
                      background: '#ecf4fa'
                    }" :row-class-name="onTableRowClassName">
                      <el-table-column prop="questionName" label="问题类型" align="center">
                      </el-table-column>
                      <el-table-column prop="proTypeDoneCount" label="已完成(个)" align="center">
                      </el-table-column>
                      <el-table-column prop="proTypeUnDoneCount" label="未完成(个)" align="center">
                      </el-table-column>
                      <el-table-column prop="proTypeRate" label="完成率(%)" align="center">
                      </el-table-column>
                    </el-table>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
<script>
import checkedTree from "@/views/deliveryManagement/components/checkedTree"

import * as echarts from 'echarts';

import { problemByType } from '@/api/problemFankuiStatustics/index'

import axios from 'axios'

import { getToken } from '@/utils/auth'

export default {
  name: "problemFankuiStatistics",
  components: {
    checkedTree
  },
  data() {
    return {
      postId: '',//楼盘id
      //data: [
      //  //this.dataValueStartTime(), this.dataValueEndTime()
      //  new Date().getFullYear()
      //],
      data: this.dataValueStartTime(),
      formUrl: process.env.VUE_APP_FORM_API + 'deliveryForm/problemByType',
      getProblemByTypeList: [],//获取问题类型
      getProblemStatsList: [],//获取问题汇总
      getProblemByTypeNoData: false,
      getProblemStatsNoData: false,

      combinedDataList: [], // 新增：合并后的数据列表

      //problemTotal: [],//问题汇总
      //appointmentProgressList: [],//交房预约和进度
      satisfactionList: [],//交房满意度
      feedbackList: [],//入住反馈统计

      activeNames: [],
      deptFlag: '',

      deptBuildId: '',//部门id


    }
  },
  mounted() {
    //this.echartsInit();
    //this.echartsPieInit();
    //this.echartsPieInit2();
    //this.getProblemByType()
    //this.getProblemStats()
    window.addEventListener('resize', this.handleResize);
  },
  methods: {
    handleChange() { },
    onTableRowClassName(row, rowIndex) {
      if (rowIndex % 2 == 0) { return 'statistics-warning-row'; } else { return ''; }
    },
    async getApi() {
      if (this.deptFlag === '1') {
        this.postId = this.deptBuildId
      } else if (this.deptFlag === '0') {
        this.postId = this.postId
      }
      try {
        // 并行获取所有数据
        await Promise.all([
          this.getProblemStatslist(),
          this.getProblemByTypeListMe(),
          this.getSatisfactionList(),
          //this.getFeedbackList(),
          this.getAppointmentProgressList()
        ]);

        // 合并所有数据
        this.mergeAllData();
      } catch (error) {
        console.error('API请求错误:', error);
        this.$message.error('API请求错误');

        //uni.showToast({
        //  title: '数据加载失败',
        //  icon: 'none'
        //});
      }
    },
    mergeAllData() {
      // 1. 收集所有出现过的 postId
      const allPostIds = new Set();

      // 从各个数据源收集 postId
      //this.problemTotal.forEach(item => allPostIds.add(item.postId));
      this.satisfactionList.forEach(item => allPostIds.add(item.postCode));
      //this.feedbackList.forEach(item => allPostIds.add(item.postId));
      //this.appointmentProgressList && this.appointmentProgressList.length > 0 && this.appointmentProgressList.forEach(item => allPostIds.add(item.postId));
      this.groupedTableData && this.groupedTableData.length > 0 && this.groupedTableData.forEach(item => allPostIds.add(item.postId));

      this.getProblemStatsList && this.getProblemStatsList.length > 0 && this.getProblemStatsList.forEach(item => allPostIds.add(item.postId));

      //console.log('allPostIds', allPostIds);
      //return
      // 2. 为每个 postId 创建合并数据对象
      this.combinedDataList = Array.from(allPostIds).map((postId, index) => {
        const mergedItem = {
          postId,
          name: this.getNameByPostId(postId) || '——'
        };
        // 合并问题汇总数据
        const problemStats = this.problemTotal.find(item => item.postId === postId);
        if (problemStats) Object.assign(mergedItem, problemStats);

        // 合并满意度数据
        const satisfaction = this.satisfactionList.find(item => item.postCode === postId);
        if (satisfaction) {
          mergedItem.jiaofuSum = satisfaction.jiaofuSum;//交付问卷总数量（有效+无效）
          mergedItem.painming = satisfaction.painming; //排名
          mergedItem.averageScore = satisfaction.jiaofuAvg; //交付平均分
          mergedItem.zongCount = satisfaction.zongCount //总评分
          mergedItem.wuxiaoWenjuansum = satisfaction.jiaofuSumYouxiao //有效问卷数
          mergedItem.invalidAverageScore = satisfaction.jiaofuyouxiaoAvg //有效平均分
          mergedItem.problemCountRZ = satisfaction.ruzhuSum;//入住问卷数量
          mergedItem.averageScoreRZ = satisfaction.ruzhuAvg;//入住平均分
          mergedItem.invalidProblemCountRZ = satisfaction.ruzhuSumYouxiao //入住有效问卷数
          mergedItem.invalidAverageScoreRZ = satisfaction.ruzhuyouxiaoAvg;
        }
        //this.getProblemByTypeNoData = true
        // 合并问题类型数据
        const problemType = this.groupedTableData.find(item => item.postId === postId);
        if (problemType) {
          const categories = [];
          const values = [];
          const weiwancheng = []
          mergedItem.problemTypeTableData = problemType.items || [];
          mergedItem.hasData = true
          //this.getProblemByTypeNoData = false
          problemType.items.forEach((item, index) => {
            categories.push(item.questionName);
            values.push(Number(item.proTypeDoneCount));
            weiwancheng.push(Number(item.proTypeUnDoneCount))
          })
          //this.$nextTick(() => {
          setTimeout(() => {
            const chartId = 'detailRightTopEcharts' + index
            this.echartsInit(chartId, categories, values, weiwancheng);
          }, 20)

          //})
        }
        //const problemStatsGW = this.getProblemStatsList.find(item => item.postId === postId);
        //this.getProblemStatsNoData = true
        //if (problemStatsGW) {
        //  this.getProblemStatsNoData = false
        //  const proTypeRate = []
        //  const gongcheng = []
        //  proTypeRate.push({ value: problemStatsGW.doneProCount, name: '已完成数' }, { value: problemStatsGW.unDoneProCount, name: '未完成数' })
        //  gongcheng.push({ value: problemStatsGW.gclProCount, name: '工程' }, { value: problemStatsGW.wyProCount, name: '物业' })
        //  this.$nextTick(() => {
        //    this.echartsPieInit(proTypeRate);

        //    this.echartsPieInit2(gongcheng);
        //  })
        //  //mergedItem.gclProCount = problemStats.gclProCount;
        //  //mergedItem.gclProDoneCount = problemStats.gclProDoneCount;
        //  //mergedItem.gclProUnDoneCount = problemStats.gclProUnDoneCount;
        //  //mergedItem.gclProRate = problemStats.gclProRate;
        //}
        return mergedItem;
      });
      console.log('合并数据', this.combinedDataList);
      if (this.combinedDataList.length === 0) {
        this.combinedDataList = [{
          name: '-'
        }]
      }
    },
    // 修改各个数据获取方法，只负责设置自己的数据
    getProblemStatslist() {
      return new Promise((resolve) => {
        const json = this.deptFlag === '0' ? { year: Number(this.data), postCode: this.postId } : { year: Number(this.data), deptCode: this.deptBuildId }
        const formUrl = process.env.VUE_APP_FORM_API + 'deliveryForm/getProblemStats'
        axios.post(formUrl, json, { headers: { 'Authorization': 'Bearer ' + getToken() } }).then(res => {
          if (res.data.code === 200) {
            //this.getProblemStatsNoData = false
            if (res.data.data.length > 0) {
              this.problemTotal = res.data.data

              const proTypeRate = []
              const gongcheng = []
              this.getProblemStatsList = res.data.data
              //this.getProblemStatsList.forEach(item => {
              //  proTypeRate.push({ value: item.doneProCount, name: '已完成数' }, { value: item.unDoneProCount, name: '未完成数' })
              //  gongcheng.push({ value: item.gclProCount, name: '工程' }, { value: item.wyProCount, name: '物业' })
              //})
              //this.$nextTick(() => {
              //  this.echartsPieInit(proTypeRate);

              //  this.echartsPieInit2(gongcheng);
              //})
            } else {
              this.problemTotal = []
              this.getProblemStatsList = []
              //this.getProblemStatsNoData = true
            }

          }
          resolve();
        }).catch(() => resolve());
        //this.$u.api.getProblemStats(json).then((res) => {
        //  this.problemTotal = res.code === 200 ? (res.data || []) : [];
        //  resolve();
        //}).catch(() => resolve());
      });
    },
    getProblemByTypeListMe() {
      return new Promise((resolve) => {
        const json = this.deptFlag === '0' ? { year: Number(this.data), postCode: this.postId } : { year: Number(this.data), deptCode: this.deptBuildId }
        const formUrl = process.env.VUE_APP_FORM_API + 'deliveryForm/problemByType'
        axios.post(formUrl, json, { headers: { 'Authorization': 'Bearer ' + getToken() } }).then(res => {
          if (res.data.code === 200) {
            //this.getProblemByTypeNoData = true
            if (res.data.data.length > 0) {
              this.tableData = res.data.data;
              //this.getProblemByTypeNoData = false
              const categories = [];
              const values = [];
              const weiwancheng = []
              this.getProblemByTypeList = res.data.data
              //this.getProblemByTypeList.forEach(item => {
              //  categories.push(item.questionName);
              //  values.push(Number(item.proTypeDoneCount));
              //  weiwancheng.push(Number(item.proTypeUnDoneCount))
              //  //proTypeRate.push({value: item.proTypeRate, name: item.questionName})
              //})
              //this.$nextTick(() => {
              //  this.echartsInit(categories, values, weiwancheng);
              //})

              // 分组处理
              const groupedData = this.tableData.reduce((acc, item) => {
                if (!acc[item.postId]) {
                  acc[item.postId] = { postId: item.postId, name: item.name, items: [] };
                }
                acc[item.postId].items.push(item);
                return acc;
              }, {});

              this.groupedTableData = Object.values(groupedData);
            } else {

              this.tableData = []
              this.getProblemByTypeList = []
              this.groupedTableData = []
              //this.getProblemByTypeNoData = true

            }

          }

          //console.log('this.groupedTableData',this.groupedTableData);
          resolve();
        }).catch(() => resolve());
        //this.$u.api.problemByType(json).then((res) => {
        //  this.tableData = res.data || [];

        //  // 分组处理
        //  const groupedData = this.tableData.reduce((acc, item) => {
        //    if (!acc[item.postId]) {
        //      acc[item.postId] = { postId: item.postId, name: item.name, items: [] };
        //    }
        //    acc[item.postId].items.push(item);
        //    return acc;
        //  }, {});

        //  this.groupedTableData = Object.values(groupedData);
        //  //console.log('this.groupedTableData',this.groupedTableData);
        //  resolve();
        //}).catch(() => resolve());
      });
    },
    getSatisfactionList() {
      return new Promise((resolve) => {
        ///build/houseInfo/countQuestionnaire
        const formUrl = process.env.VUE_APP_BASE_API + 'build/houseInfo/countQuestionnaire'
        const json = this.deptFlag === '0' ? { year: Number(this.data), postCode: this.postId } : { year: Number(this.data), deptCode: this.deptBuildId }
        axios.post(formUrl, json, { headers: { 'Authorization': 'Bearer ' + getToken() } }).then(res => {
          if (res.data.code === 200) {
            this.satisfactionList = res.data.data || [];
          }
          resolve();
        }).catch(() => resolve());
        //this.$u.api.getSatisfaction({ year: Number(this.startDateStr) }).then((res) => {
        //  this.satisfactionList = res.code === 200 ? (res.data || []) : [];
        //  resolve();
        //}).catch(() => resolve());
      });
    },

    getAppointmentProgressList() {
      return
      return new Promise((resolve) => {
        const formUrl = process.env.VUE_APP_BASE_API + 'delivery/appointmentProgressWeb'
        axios.post(formUrl, { year: Number(this.data), postId: this.postId }, { headers: { 'Authorization': 'Bearer ' + getToken() } }).then(res => {
          if (res.data.code === 200) {
            this.appointmentProgressList = res.data.data || [];
          }
          resolve();
        }).catch(() => resolve());
        //this.$u.api.appointmentProgress({ year: Number(this.data) }).then((res) => {
        //  this.appointmentProgressList = res.code === 200 ? (res.data || []) : [];
        //  resolve();
        //}).catch(() => resolve());
      });
    },
    handleResize() {
      // 获取所有图表实例并重新调整大小
      //const charts = this.deptFlag === '1' ? [
      //  echarts.getInstanceByDom(document.getElementById('detailRightTopEcharts')),
      //  echarts.getInstanceByDom(document.getElementById('detailRightbottomLeftEcharts')),
      //  echarts.getInstanceByDom(document.getElementById('detailRightbottomRightEcharts'))
      //] : [
      //  echarts.getInstanceByDom(document.getElementById('detailRightTopEchartsTwo')),
      //  echarts.getInstanceByDom(document.getElementById('detailRightbottomLeftEchartsTwo')),
      //  echarts.getInstanceByDom(document.getElementById('detailRightbottomRightEchartsTwo'))
      //];
      const charts = [
        echarts.getInstanceByDom(document.getElementById('detailRightTopEcharts0')),
        echarts.getInstanceByDom(document.getElementById('detailRightTopEcharts1'))
      ]

      charts.forEach(chart => {
        if (chart) {
          chart.resize();
        }
      });
    },
    // 辅助方法：从任意数据源获取楼盘名称
    getNameByPostId(postId) {
      // 按优先级尝试不同数据源
      const dataSources = [
        //this.appointmentProgressList,
        //this.problemTotal,
        this.satisfactionList,
        //this.feedbackList,
        this.groupedTableData,
        this.getProblemStatsList
      ];

      for (const source of dataSources) {
        if (source === this.satisfactionList) {
          const item = source.find(i => i.postCode === postId);
          if (item && item.title) return item.title;
        } else {
          const item = source.find(i => i.postId === postId);
          if (item && item.name) return item.name;
        }
      }

      return null;
    },
    // 搜索某个楼盘对应的交房进度跟踪
    searchProcess(val) {
      this.postId = val.id

      this.deptFlag = val.deptFlag
      //this.getProblemByType()
      //this.getProblemStats()
      this.getApi()
    },
    datePickerChange(val) {
      if (val === '') {
        this.data = this.dataValueStartTime()
      }
    },
    echartsInit(chartId, questionName, proTypeDoneCount, weiwancheng) {
      //var chartDom = this.deptFlag === '1' ? document.getElementById(chartId) : document.getElementById('detailRightTopEchartsTwo');

      var chartDom = document.getElementById(chartId); // 直接使用传入的chartId
      if (!chartDom) {
        console.error('找不到图表容器:', chartId);
        return;
      }

      var myChart = echarts.init(chartDom);
      var option;

      option = {
        tooltip: {
          trigger: 'axis',
        },

        xAxis: [
          {
            type: 'category',
            data: questionName,
            axisPointer: {
              type: 'shadow'
            },
            axisLabel: {
              rotate: 30, // 倾斜角度，正值表示逆时针旋转
              interval: 0, // 强制显示所有标签
              // 如果需要文字换行可以添加以下配置
              // formatter: function(value) {
              //   return value.split('').join('\n'); // 每个字符换行
              //   // 或者按特定长度换行
              //   // return value.match(/.{1,3}/g).join('\n');
              // }
            }
          }
        ],
        yAxis: [
          {
            type: 'value',
            name: '已完成',
            min: 0,
            minInterval: 1,
            //max: function (value) {
            //  // 计算最大值
            //  const maxValue = Math.max(...proTypeDoneCount);
            //  //if (maxValue === 0) return 5;
            //  //if (maxValue < 5) {
            //  //  return 5;
            //  //} else if (maxValue < 50) {
            //  //  return Math.ceil(maxValue / 10) * 10;
            //  //} else if (maxValue < 100) {
            //  //  return Math.ceil(maxValue / 20) * 20;
            //  //} else {
            //  //  return Math.ceil(maxValue / 50) * 50;
            //  //}
            //  if (maxValue < 10) {
            //    return 10;
            //  } else if (maxValue < 50) {
            //    return Math.ceil(maxValue / 10) * 10;
            //  } else if (maxValue < 100) {
            //    return Math.ceil(maxValue / 20) * 20;
            //  } else {
            //    return Math.ceil(maxValue / 50) * 50;
            //  }
            //},
            splitNumber: 5,

          },
          {
            type: 'value',
            name: '未完成',
            min: 0,
            minInterval: 1,
            //max: function (value) {
            //  // 计算最大值
            //  const maxValue = Math.max(...weiwancheng);
            //  if (maxValue === 0) return 10;
            //  if (maxValue < 10) {
            //    return 10;
            //  } else if (maxValue < 50) {
            //    return Math.ceil(maxValue / 10) * 10;
            //  } else if (maxValue < 100) {
            //    return Math.ceil(maxValue / 20) * 20;
            //  } else {
            //    return Math.ceil(maxValue / 50) * 50;
            //  }
            //},
            splitNumber: 5,
            //axisLabel: {
            //  formatter: '{value} °C'
            //}
          }
        ],
        series: [
          {
            name: '已完成',
            type: 'bar',
            yAxisIndex: 0,  //指定需要使用的Y轴
            tooltip: {
              //valueFormatter: function (value) {
              //  return value + ' ml';
              //}
            },
            data: proTypeDoneCount
          },
          {
            name: '未完成',
            type: 'bar',
            yAxisIndex: 1,  //指定需要使用的Y轴
            tooltip: {
              //valueFormatter: function (value) {
              //  return value + ' ml';
              //}
            },
            data: weiwancheng
          },
        ],

        //dataZoom: [
        //  {
        //    type: "slider",
        //    xAxisIndex: 0,
        //    height: 1,
        //    bottom: 2, // 调整距离底部的距离（单位px）
        //    backgroundColor: "#fff", // 组件的背景颜色
        //    borderColor: "#D4D1D1", // 边框颜色
        //    showDetail: false, // 是否显示detail，即拖拽时候显示详细数值信息
        //    showDataShadow: false, // 是否在 dataZoom-silder 组件中显示数据阴影。数据阴影可以简单地反应数据走势。
        //  },
        //  {
        //    type: "inside",
        //    xAxisIndex: [0],
        //    zoomOnMouseWheel: false, // 不按任何功能键，鼠标滚轮能触发缩放
        //    moveOnMouseWheel: false, // 不按任何功能键，鼠标移动能触发数据窗口平移
        //  },
        //],
      };
      option && myChart.setOption(option);
      // 响应窗口调整大小
      window.addEventListener('resize', function () {
        myChart.resize();
      });
      //console.log('echarts初始化成功', option);
    },
    echartsPieInit(proTypeRate) {
      var chartDom = this.deptFlag === '1' ? document.getElementById('detailRightbottomLeftEcharts') : document.getElementById('detailRightbottomLeftEchartsTwo');
      var myChart = echarts.init(chartDom);
      var option;

      option = {
        tooltip: {
          trigger: 'item'
        },
        legend: {
          orient: 'vertical',
          left: 'left'
        },
        series: [
          {
            type: 'pie',
            radius: ['30%', '40%'],
            avoidLabelOverlap: false,
            data: proTypeRate
          }
        ]
      };
      option && myChart.setOption(option);
    },
    echartsPieInit2(gongcheng) {
      var chartDom = this.deptFlag === '1' ? document.getElementById('detailRightbottomRightEcharts') : document.getElementById('detailRightbottomRightEchartsTwo');
      var myChart = echarts.init(chartDom);
      var option;

      option = {

        tooltip: {
          trigger: 'item'
        },
        legend: {
          orient: 'vertical',
          left: 'left'
        },
        series: [
          {
            type: 'pie',
            radius: '40%',
            data: gongcheng,
            emphasis: {
              itemStyle: {
                shadowBlur: 10,
                shadowOffsetX: 0,
                shadowColor: 'rgba(0, 0, 0, 0.5)'
              }
            }
          }
        ]
      };

      option && myChart.setOption(option);
    },
    //查询
    query() {
      //this.getProblemByType()
      //this.getProblemStats()

      this.getApi()

      //this.searchDeptProcess()
    },
    //重置
    reset() {
      //this.data = [
      //  this.dataValueStartTime(), this.dataValueEndTime()
      //]

      //this.getProblemByType()
      //this.getProblemStats()
      this.data = this.dataValueStartTime()
      this.getApi()
    },
    dataValueStartTime() {
      const now = new Date();
      const year = now.getFullYear();
      return `${year}`;
      //const month = String(now.getMonth()).padStart(2, '0');
      //const day = String(now.getDate()).padStart(2, '0');
      //const hours = String(now.getHours()).padStart(2, '0');
      return `${year}-${month}-${day}`;
    },
    dataValueEndTime() {
      const now = new Date();
      const year = now.getFullYear();
      const month = String(now.getMonth() + 1).padStart(2, '0');
      const day = String(now.getDate()).padStart(2, '0');
      const hours = String(now.getHours() - 1).padStart(2, '0');

      return `${year}-${month}-${day}`;
    },

    //获取问题类型//该方法没有使用
    getProblemByType() {
      return
      const url = this.formUrl;
      axios.post(url, {
        postId: this.postId,
        year: Number(this.data),
        //startTime: this.data[0] + ' 00:00:00',
        //endTime: this.data[1] + ' 23:59:59'
      }, {
        headers: {
          'Authorization': 'Bearer ' + getToken(),
        }
      })
        .then(res => {
          this.getProblemByTypeNoData = false
          if (res.data.code === 200) {
            this.getProblemByTypeNoData = false
            if (res.data.data.length > 0) {
              this.getProblemByTypeNoData = false
              const categories = [];
              const values = [];
              const weiwancheng = []
              this.getProblemByTypeList = res.data.data
              this.getProblemByTypeList.forEach(item => {
                categories.push(item.questionName);
                values.push(Number(item.proTypeDoneCount));
                weiwancheng.push(Number(item.proTypeUnDoneCount))
                //proTypeRate.push({value: item.proTypeRate, name: item.questionName})
              })
              this.$nextTick(() => {
                this.echartsInit(categories, values, weiwancheng);
              })

              //this.echartsPieInit(proTypeRate);
            } else {
              this.getProblemByTypeNoData = true
              this.getProblemByTypeList = []
            }
          }
        })
        .catch(error => {
          console.error('请求出错:', error);
        });
    },

    //获取问题汇总统计
    getProblemStats() {
      const url = process.env.VUE_APP_FORM_API + 'deliveryForm/getProblemStats';

      let json = {}
      if (this.deptFlag == '1') {
        json = {
          postCode: this.postId,
          year: Number(this.data),
        }
      } else {
        json = {
          deptCode: this.deptBuildId,
          year: Number(this.data),
        }
      }
      axios.post(url, json, {
        headers: {
          'Authorization': 'Bearer ' + getToken(),
        }
      })
        .then(res => {
          this.getProblemStatsNoData = false
          if (res.data.code === 200) {
            this.getProblemStatsNoData = false
            if (res.data.data.length > 0) {

              const proTypeRate = []
              const gongcheng = []
              this.getProblemStatsList = res.data.data
              this.getProblemStatsList.forEach(item => {
                proTypeRate.push({ value: item.doneProCount, name: '已完成数' }, { value: item.unDoneProCount, name: '未完成数' })
                gongcheng.push({ value: item.gclProCount, name: '工程' }, { value: item.wyProCount, name: '物业' })
              })
              this.$nextTick(() => {
                this.echartsPieInit(proTypeRate);

                this.echartsPieInit2(gongcheng);
              })

            } else {
              this.getProblemStatsNoData = true
              this.getProblemStatsList = []
            }
          }
          //console.log('接口返回的数据:', response.data);
        })
        .catch(error => {
          console.error('请求出错:', error);
        });
    },

    searchDeptProcess(val) {
      this.deptFlag = val.deptFlag
      this.deptBuildId = val.id
      this.getApi()
      //this.getProblemByTypeNoData = true
      //this.getProblemStatsNoData = true
      //this.combinedDataList = [{
      //  name: '暂无数据'
      //}]

    }
  }
}
</script>

<style lang="scss" scoped>
.deliveryProgress {
  display: flex;
  justify-content: flex-start;
  width: 100%;
  height: 100vh;
  background-color: #f5f8f8;

  .searchLeft {
    box-sizing: border-box;
    padding: 20px;
    flex: 0 0 auto;
    /* 宽度由内容决定 */

    /* 添加滚动条相关样式 */
    height: 100%;
    /* 继承容器高度 */
    overflow-y: auto;

    /* 可选：限制最大宽度 */
    max-width: 50%;
    /* 防止过宽 */
  }

  .detailRight {
    box-sizing: border-box;
    flex: 1;
    /* 关键：占据剩余空间 */
    padding: 20px 10px;
    min-width: 0;
    /* 防止内容溢出 */
    margin-bottom: 10px;

    .topBox {
      height: 57px;

      .buttonCs {
        margin-left: 10px;
      }
    }

    .bottomBox {
      height: 683px;
      overflow-y: scroll;

      .detailRightTop {
        //height: 200px;
        margin-top: 10px;
        width: 100%;
        height: calc(50% - 20px);
        background-color: #fff;
        padding: 20px;
        border: 1px solid #fff;
        border-radius: 10px;
        //background: blue;

      }

      .detailRightTopTitle {
        font-size: 14px;
        font-weight: bold;
        height: 20px;
        margin-bottom: 10px;
        display: flex;
        align-items: center;
        text-align: center;

        p {
          margin-left: 5px;
        }
      }

      .detailRightbottom {
        display: flex;
        width: 100%;
        height: calc(50% - 20px);
        margin-top: 10px;
        justify-content: space-between;
        box-sizing: border-box;
        //border: 1px solid #fff;
        //border-radius: 10px;
        //background: #fff;

        .detailRightbottomLeft {
          width: 49.8%;
          height: 100%;
          font-size: 14px;
          font-weight: bold;
          border: 1px solid #fff;
          border-radius: 10px;
          background: #fff;
          margin-right: 10px;
          padding: 20px;
        }

        .detailRightbottomRight {
          width: 49.8%;
          height: 100%;
          font-size: 14px;
          font-weight: bold;
          border: 1px solid #fff;
          border-radius: 10px;
          background: #fff;
          padding: 20px;
        }
      }

      .handoverCockpit {
        height: 300px;
        background-color: red;
      }

      .cardContent {
        /*height: 400px;*/
        display: flex;
        flex-wrap: wrap;
        justify-content: space-between;

        .cardContentItem {
          width: 48%;
          height: 158px;
          border: 1px solid #f5f5f5;
          border-radius: 30px;
          box-shadow: 2px 2px 2px rgba(187, 187, 187, .3);
          display: flex;
          flex-direction: column;
          justify-content: center;
          align-items: center;
        }

        .cardContentItem:nth-child(1) {
          margin-bottom: 20px;
        }

        .cardContentItemNumber {
          font-weight: 800;
        }

        .cardContentItemTextTitle {
          font-weight: 800;
          font-size: 26px;
        }

      }

      .cardContentTwo {
        /*height: 400px;*/
        min-width: 1314px;
        display: flex;
        flex-wrap: nowrap;
        justify-content: space-between;
        margin-top: 10px;

        .cardContentTwoItem {
          //max-width:741.52px ;
          //width: 741.52px;
          width: 49.7%;
          padding: 22px;
          display: flex;
          flex-direction: column;
          box-sizing: border-box;
          line-height: 20px;
          background: #fff;
          border-radius: 10px;
        }

        .cardContentTwoItemNumberItem {
          display: flex;
          justify-content: space-between;
          align-items: center;

          .cardContentTwoItemNumber {
            width: 313px;
            height: 80px;
            display: flex;
            flex-wrap: nowrap;
            flex-direction: column;
            justify-content: center;
            align-items: center;
            //justify-content: space-between;
            //font-weight: 800;
            //padding-left: 28px;
            font-size: 14px;

          }
        }


        .cardContentTwoItemTextTitle {
          //font-weight: 800;
          //font-size: 26px;
          font-size: 14px;
          font-weight: bold;
          height: 20px;
          margin-bottom: 10px;
          display: flex;
          align-items: center;

          p {
            margin-left: 5px;
          }
        }

      }

      .problemSummaryTitle {
        width: 100%;
        height: auto;
        border-radius: 10px;
        margin-top: 10px;
        box-sizing: border-box;
        //padding-top: 20px;
        //background-color: #fff;

        .problemTjItem {
          width: 25%;
          height: 50px;
          display: flex;
          flex-direction: column;
          justify-content: center;
          align-items: center;
          font-size: 14px;
          line-height: 42px;
        }

        .problemCZFTJTitle {
          padding: 0 30px;
          font-size: 14px;
          font-weight: 800;
        }

        .problemTypeTjTitle {
          padding: 0 30px;
          font-size: 14px;
          font-weight: 800;
        }

        .problemTj {
          //width: 100%;
          min-width: 1314px;
          height: 93px;
          display: flex;
          justify-content: space-between;
          align-items: center;

          .problemTjItem {
            width: 25%;
            height: 71px;
            display: flex;
            flex-direction: row;
            justify-content: space-between;
            align-items: center;
            font-size: 14px;
            line-height: 42px;
            margin: 0 30px;
            border-radius: 5px;
            padding: 30px;
            box-sizing: border-box;

            .problemTjItem_item {
              display: flex;
              justify-content: center;
              align-items: center;
            }
          }
        }

        .problemSummary {
          width: 100%;
          height: auto;
          border-radius: 50px;
          border: 1px solid #f5f5f5;
          box-sizing: border-box;
        }



        .problemTjItem-first {
          border-right: 1px solid #dfdfdf;
          padding: 0 !important;
        }

        .problemCZFTJTitle {
          padding: 0 30px;
          font-size: 32px;
          font-weight: 800;
        }

        .problemTypeTjTitle {
          padding: 0 30px;
          font-size: 32px;
          font-weight: 800;
        }


        .uni-container {
          padding: 10px 22px 22px 22px;
        }



        .example-body {
          width: 30px;
          height: 30px;
          z-index: 999;
        }
      }

      .lineClass {
        width: 100%;
        height: 4px;
        background-color: #d8d8d8;
        margin: 20px 0;
      }


    }

    .bottomBoxTwo {
      height: 768px;
      overflow-y: scroll;

      .headerBottom {
        //width: 100%;
        min-width: 1314px;
        display: flex;
        justify-content: start;
        align-items: center;
        background-color: #fff;
        padding: 30px 0;
        border-radius: 10px;

        .headerBottom_items {
          width: 313px;
          height: 71px;
          background-color: #EFFAF3;
          display: flex;
          align-items: center;
          justify-content: space-between;
          //margin-top: 10px;
          //margin-bottom: 10px;
          margin: 0 30px;
          padding: 30px;
          box-sizing: border-box;
          border-radius: 5px;
          font-size: 14px;

          .headerBottom_item {
            display: flex;
            align-items: center;
            //justify-content: space-between;
            //width: 100px;
            //height: 30px;
            //border-radius: 10px;
            //background-color: #1890FF;
            //color: #fff;
            font-size: 14px;
            font-weight: 0;
            //font-weight: bold;
            //margin-right: 10px;
            cursor: pointer;
          }
        }
      }
    }
  }


  .noData {
    font-size: 14px;
    font-weight: bold;
    width: 100%;
    height: 249.5px;
    text-align: center;
    color: #6e7079;
    display: flex;
    justify-content: center;
    align-items: center
  }

  .verLine {
    width: 3px;
    height: 18px;
    background-color: #1890FF;
    float: left;
    margin-right: 6px;
    border-radius: 10px;
    border: 1px solid #1890FF;
  }

  .content {
    .problemCZFTJ {
      .problemTj_item {
        //width: 100%;
        min-width: 1314px;
        display: flex;
        flex-direction: row;
        justify-content: space-between;

        .problemTjZong {
          width: 50%;
          display: flex;
          flex-wrap: wrap;

          .problemTjItem {
            width: 50%;
            height: 96px;
            display: flex;
            flex-direction: row;
            justify-content: start;
            padding: 0 6px;
            box-sizing: border-box;

            .problemTjItem_items {
              //height: 50px;
              display: flex;
              flex-direction: column;
              margin-left: 15px;

              span:nth-child(2) {
                font-size: 14px;
                font-weight: bold;
              }
            }
          }
        }
      }
    }
  }

  .collapse-container {
    padding-bottom: 10px;
    max-height: 78vh;
    overflow-y: auto;
  }

  .collapse_Bigbox {
    //max-height: 768px;
    //overflow-y: auto;
    //border: 1px solid #ebeef5;
    border-radius: 8px;
    overflow: hidden;
  }

  .collapse_class {
    margin-bottom: 15px;

    .el-collapse-item__headers {
      //height: 168px;
      //width: 100%;
      width: 1314px;
      border-bottom: none;
      display: flex;
      //align-items: center;
      flex-direction: column;
      //padding-left: 20px;

      .headerBottom {
        width: 100%;
        display: flex;
        justify-content: start;
        align-items: center;

        .headerBottom_items {
          width: 313px;
          height: 71px;
          background-color: #EFFAF3;
          display: flex;
          align-items: center;
          justify-content: space-between;
          //margin-top: 10px;
          //margin-bottom: 10px;
          margin: 0 30px;
          padding: 30px;
          box-sizing: border-box;
          border-radius: 5px;
          font-size: 14px;

          .headerBottom_item {
            display: flex;
            align-items: center;
            //justify-content: space-between;
            //width: 100px;
            //height: 30px;
            //border-radius: 10px;
            //background-color: #1890FF;
            //color: #fff;
            font-size: 14px;
            font-weight: 0;
            //font-weight: bold;
            //margin-right: 10px;
            cursor: pointer;
          }
        }
      }
    }
  }

  .collapse_class:last-child {
    margin-bottom: 0;
  }

}
</style>
<style scoped>
::v-deep .el-collapse-item__wrap {
  background-color: transparent;
}

::v-deep .el-collapse-item__header {
  height: 174px;
  /*line-height: 10px;*/
  /*padding: 10px;*/
  /*border-bottom: none;
  align-items: normal;
  border-radius: 10px;*/
}

.el-collapse {
  border-bottom: none;
  /*border-radius: 10px;*/
  border-top: none;
}

::v-deep .el-collapse-item__arrow {
  display: flex;
  align-items: center;
}

::v-deep .el-collapse-item__content {
  padding-bottom: 0px;
}
</style>