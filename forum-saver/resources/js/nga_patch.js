{
    //翻页按钮//===================
    /**
     * @param {*} o
     * @param {*} p p[1] 最后页 为0时显示 可能有下1页   小于0显示 可能有下p[1]*-1页
     * @param {*} opt &1auto &2不显示向前部分的翻页 &4不显示向后部分的翻页 &8显示下一页加载 &16显示前一页加载 &32超小
     * @returns {*}
     */

    commonui.pageBtn = function (o, p, opt) {

        if (__SETTING.bit & 134217728)
            return


        if (typeof o == 'string') {
            //if(document.currentScript)
            //	o = document.currentScript.parentNode
            //else
            o = document.getElementById(o).parentNode
        }

        if (opt & 1)
            opt |= o.id == 'pagebtop' ? 16 : 8

        /*
        if((opt&1) && this.pageBtn.cache && this.pageBtn.cache.nodeName=='TABLE'){
            o.innerHTML=''
            o.appendChild(	this.pageBtn.cache.cloneNode(1)	)
            var x = o.getElementsByTagName('a')
            for(var i=0;i<x.length;i++){
                if(x[i].name=='topage')
                    _$(x[i])._.on('click',function(e){commonui.jumpToForm(e,p[3],p[2],p[1])})
                }
            return
            }
        */

        var hl, hln, hlp, s, bit = __SETTING.bit, more = (bit & 8) ? 1 : ((bit & 4) ? 3 : 5), url = p[0], max = p[1], postPerPage = p[3], e = max
            , cur = p[2] //当前最小的一页
            , cur2 = p[4]//当前最大的一页 仅在连续加载时

        if (cur2) {
            if (cur2 < cur) {
                cur = cur2
                cur2 = p[2]
            }
        }
        else
            cur2 = cur

        var curcur = ((opt & 4) ? cur : cur2)

        if (bit & 16)//small
            opt |= 32

        if (window.__APPEMBED) {
            if (window.__LOADERREAD) {
                if (!__LOADERREAD.maxUrlInit) {
                    __LOADERREAD.maxUrlInit = 1
                    var uu = location.protocol + '//' + location.host + location.port + '/'
                    if (window.__CURRENT_TID) {
                        if (window.__CURRENT_PID)
                            uu += 'read.php?tid=' + __CURRENT_TID + '&pid=' + __CURRENT_PID
                        else
                            uu += 'read.php?tid=' + __CURRENT_TID + '&page=' + cur
                    }
                    else if (window.__CURRENT_STID)
                        uu += 'thread.php?stid=' + __CURRENT_STID + '&page=' + cur
                    else if (window.__CURRENT_FID)
                        uu += 'thread.php?fid=' + __CURRENT_FID + '&page=' + cur
                    __LOADERREAD.maxUrl = uu
                }
            }
        }

        s = opt & 2 ? cur2 - more : cur - more
        if (s < 1)
            s = 1
        if (commonui.htmlLoader)
            hl = hlp = hln = function () { }
        else if (commonui.loadReadHidden) {
            if (__SETTING.uA[6] & 3)
                hl = function (e) { commonui.loadReadHidden(this.value, 1); commonui.cancelBubble(e); commonui.cancelEvent(e) }
            hlp = function (e) { commonui.cancelBubble(e); commonui.cancelEvent(e); commonui.loadReadHidden(0, 4) }
            hln = function (e) { commonui.cancelBubble(e); commonui.cancelEvent(e); commonui.loadReadHidden(0, 2) }
        }
        else
            hl = hlp = hln = function () { }

        if (max < 1) {//可能有下页
            if (max < 0)//显示指定的页数
                e = cur2 - max
            else//显示下一页
                e = cur2 + 1
        }

        if (opt & 4) {
            if (e > cur + more)
                e = cur + more
        }
        else {
            if (e > cur2 + more)
                e = cur2 + more
        }

        if (e < s)
            return

        // url替换为本地路径
        url_local = url.split('?')[1]
        var oo = this.stdBtns()
        for (var i = s; i <= e; i++) {
            //if((opt&2) && i<cur)
            //	continue
            //if((opt&4) && i>cur)
            //	continue
            if (i == cur) {
                if (opt & 16) {
                    if (i > 1) {
                        var pp = _$('/a',
                            'href', url_local + '&page=' + (i - 1) + '.html',
                            'value', i - 1,
                            'innerHTML', '&lt;',
                            'title', '加载上一页',
                            'className', 'uitxt1',
                            'style', 'display:none',
                            '_useloadread', (opt & 32) ? 1 : 9
                        )
                        oo._.__add(pp)
                        //if(!this.pageBtn.continuePrevO || this.pageBtn.continuePrevO._page>nn)
                        this.pageBtn.continuePrevO = pp
                        if ((opt & 32) == 0)
                            pp.style.display = ''
                    }
                    else
                        this.pageBtn.continuePrevO = null
                }
            }


            var sc = (opt & 32) && i == curcur, is, sa = ''

            if (i == max) {
                if (i == curcur)
                    is = '\u00a0' + i + '\u0324\u00a0'
                else
                    is = '\u00a0' + i + '\u0323\u00a0'
            }
            else if (i == curcur)
                is = '\u00a0' + i + '\u0323\u00a0'
            else
                is = '\u00a0' + i + '\u00a0'

            if (opt & 32) {
                if (i > 999)
                    is = "<span style='display:inline-block;line-height:1em;vertical-align:-50%;'>" + is.replace(/(\d{2})/, '$1\u00a0<br/>\u00a0') + "</span>\u200b"
                else if (i > 99)
                    is = is.replace(/\u00a0/g, '')
            }

            oo._.__add(_$('/a',
                'href', url_local + '&page=' + i + '.html',
                'innerHTML', is,
                'style', sa,
                'className', (i > cur2 ? (max > 1 ? 'uitxt1' : 'silver') : (i < cur ? 'uitxt1' : ((i == cur || i == cur2) ? 'invert' : 'silver'))),
                'title', (i > cur && max <= 1 ? '可能有第' + i + '页' : (i == max ? '最后页' : '')),
                'value', i,
                '_useloadread', sc ? (16 | 32) : 1,
                'onclick', sc ? function (e) { commonui.jumpToForm(e, postPerPage, cur, max); commonui.cancelBubble(e); return commonui.cancelEvent(e) } : null
            ))
            if (i == cur2) {
                if (opt & 8) {
                    if (i < e) {
                        var nn = _$('/a',
                            'href', url + '&page=' + (i + 1) + '.html',
                            'value', i + 1,
                            'innerHTML', '&gt;',
                            'title', '加载下一页',
                            'className', 'uitxt1',
                            'style', 'display:none',
                            '_useloadread', 9
                        )
                        oo._.__add(nn)
                        //if(!this.pageBtn.continueNextO || this.pageBtn.continueNextO._page<nn)
                        this.pageBtn.continueNextO = nn
                        if ((opt & 32) == 0)
                            nn.style.display = ''
                    }
                    else
                        this.pageBtn.continueNextO = null
                }
            }


        }
        if (cur > 1 && (bit & 4) == 0 && (opt & 2) == 0) {
            oo._.__ins(
                _$('/a',
                    'href', url_local + '&page=' + (cur - 1) + '.html',
                    'innerHTML', '前页',
                    'title', '上一页',
                    'className', 'uitxt1',
                    'value', (cur - 1),
                    '_useloadread', 1,
                    hl ? { onclick: hl } : null
                ), 1
            )
        }
        if (s > 1 && (opt & 32) == 0) {
            oo._.__ins(
                _$('/a',
                    'href', url_local + '&page=1.html',
                    'innerHTML', bit & 8 ? '首' : '首页',
                    'title', '第一页',
                    'className', 'uitxt1',
                    'value', 1,
                    '_useloadread', 1,
                    hl ? { onclick: hl } : null
                ), 1
            )
        }
        if (cur2 < max && (bit & 4) == 0 && (opt & 4) == 0) {
            oo._.__add(
                _$('/a',
                    'href', url_local + '&page=' + (cur2 + 1) + '.html',
                    'innerHTML', '后页',
                    'title', '下一页',
                    'className', 'uitxt1',
                    'value', (cur2 + 1),
                    '_useloadread', 1,
                    hl ? { onclick: hl } : null
                ), 1
            )
        }
        if (e < max && (url.substr(0, 9) == '/read.php' || __GP.admincheck) && (opt & 32) == 0) {
            oo._.__add(
                _$('/a',
                    'href', url_local + '&page=' + max + '.html',
                    'innerHTML', bit & 8 ? '尾' : '末页',
                    'title', '最后页 第' + max + '页',
                    'className', 'uitxt1',
                    'value', max,
                    '_useloadread', 1,
                    hl ? { onclick: hl } : null
                )
            )
        }
        if (max != 1) {
            oo._.__add(
                _$('/a',
                    'href', 'javascript:void(0)',
                    'innerHTML', '到',
                    'name', 'topage',
                    'title', '到指定的页数',
                    'className', 'uitxt1',
                    'onclick', function (e) { commonui.jumpToForm(e, postPerPage, cur, max) }
                )
            )
        }

        if (window.__APPEMBED)
            this.stdBtnsAppStyleFix(oo)

        o.innerHTML = ''

        if (!oo._.__length)
            return

        o.appendChild(oo)
        if (oo._.__vml)
            oo._.__vml()
        //this.pageBtn.cache =o.firstChild

    }//fe

    commonui.pageBtn.continueNext = function () {
        if (this.continueNextO)
            __NUKE.fireEvent(this.continueNextO, 'click')
    }//

    commonui.pageBtn.continuePrev = function () {
        if (this.continuePrevO) {
            __NUKE.fireEvent(this.continuePrevO, 'click')
            if ((this.continuePrevO._useloadread & 8) == 0)
                this.scrollEndOnce = 1
            return true
        }
        else if (window.__CURRENT_TID) {
            var o = _$('/a',
                'href', location.protocol + '//' + location.host + location.port + '/read.php?tid=' + __CURRENT_TID + '&pid=' + window.__CURRENT_PID,
                '_useloadread', 1,
                'style', 'display:none'
            )
            document.body.appendChild(o)
            __NUKE.fireEvent(o, 'click')
        }
        return false
    }//

    //翻页跳转 select==============
    commonui.jumpToForm = function (e, postPerPage, cp, mp) {

        var min = cp - 10, max = cp + 10, $ = _$, s = $('/select'), co = $('/div', 'className', 'ltxt  group', 'style', 'max-width:45em')
            , to = function (p, lo) {
                var w = window
                if (w.__APPEMBED) {
                    var q = { page: p }
                    if (w.__CURRENT_AUTHORID)
                        q.authorid = w.__CURRENT_AUTHORID
                    if (w.__CURRENT_TID) {
                        if (w.__CURRENT_OPT)
                            q.opt = w.__CURRENT_OPT
                        q.tid = w.__CURRENT_TID
                        __doAction.appDoSync('readPost', q)
                    }
                    else if (w.__CURRENT_STID) {
                        q.stid = w.__CURRENT_STID
                        __doAction.appDoSync('readForum', q)
                    }
                    else if (w.__CURRENT_FID) {
                        q.fid = w.__CURRENT_FID
                        __doAction.appDoSync('readForum', q)
                    }
                    return
                }
                var l = w.location, h = l.protocol + '//' + l.host + l.pathname + (l.search.replace(/(?:\?|&)page=(?:e|\d+)/gi, '') + '&page=' + p).replace(/^&/, '?')
                if (lo)
                    h = h.replace(/#.+$/, '') + '#' + lo
                w.location.href = h
            }
            , ga = function (p, s, n) {
                return $('/a', 'href', 'javascript:void(0)', 'onclick', function () { to(this._p) }, '_p', p, 'innerHTML', n ? n : (p < 10 ? ('&emsp;' + p) : p), 'className', 'cell rep txtbtnx nobr disable_tap_menu block_txt_big ' + (s ? s : ''))
            }
        if (min < 1) min = 1
        if (max > mp) max = mp
        if (min > 1) {
            s._.add($('/option', 'innerHTML', 1, 'value', 1))
            co._.add(ga(1))
        }


        for (var i = min; i <= max; i++) {
            s._.add($('/option', 'innerHTML', i, 'value', i, i == cp ? 'selected' : '_null', 1))
            co._.add(ga(i, i == cp ? 'block_txt_c2' : ''))
        }

        s._.on('change', function () {
            if (!this.options[this.selectedIndex].value)
                return
            to(this.options[this.selectedIndex].value)
        })

        if (max < mp) {
            s._.add($('/option', 'innerHTML', '末页(' + mp + ')', 'value', mp))
            co._.add(ga(mp, '', '末页(' + mp + ')'))
        }

        co._.add($('/br'), $('/br'))

        if (window.__CURRENT_TID)
            co._.add('到',
                $('/input', 'size', 4),
                $('/button', 'innerHTML', ' 楼 ', 'onclick', function () {
                    var x = this.previousSibling.value | 0
                    if (x < 0) return
                    to(Math.ceil((x + 1) / postPerPage), x)
                }
                ),
                '所在的页',
                $('/br'),
                $('/br'))

        co._.add('到',
            $('/input', 'size', 4),
            $('/button', 'innerHTML', ' 页 ', 'onclick', function () {
                var x = this.previousSibling.value | 0
                if (x < 1) return
                to(x)
            }
            ),
            $('/br'),
            $('/br'),
            '到', s, ' 页 '
        )

        this.createadminwindow()
        this.adminwindow._.addContent(null)
        this.adminwindow._.addContent(co)
        this.adminwindow._.show(e)
    }//fe

    //判断翻页按钮是否折行==========
    commonui.pageBtnAdjHeight = function (o, oo) {
        //window.setTimeout(function(){
        //	if(!oo.offsetHeight || o.offsetHeight<oo.offsetHeight*1.5)return
        //	o.className = 'doublebtns'
        //	},150)
    }//fe
    commonui.getAttachBase = function (u) {
        return `https://${__ATTACH_BASE_VIEW}/attachments`
    }
    ubbcode.attach.load = function (o, nouse, a, pid, tid, authorId, postTime, id) {
        // console.log("o=", o);
        if (typeof (o) == 'string') o = $(o)

        if (id === undefined) {
            if (id = o.id.match(/(\d+)$/))
                id = 'attach' + id[1]
            else
                id = this.parent.randDigi('attach_noid', 10000)
        }
        else
            id = 'attach' + id

        this.cache[id] = { a: a, pid: pid, tid: tid, authorId: authorId, postTime: postTime, o: o, index: {} }


        for (var k = 0; k < a.length; k++) {
            if (!a[k].name) a[k].name = a[k].url.match(/[^\/]+$/)[0]
            var aa = this.parseName('/' + a[k].name)

            if (aa.type != 'zip' && aa.type != a[k].type)
                a[k].type = aa.type
            a[k].thumb = aa.thumb ? aa.thumb : (a[k].thumb | 0)
            a[k].size = aa.size ? aa.size : (a[k].size | 0)
            if (!a[k].dscp) a[k].dscp = ''
            this.cache[id].index[aa.cacheKey] = a[k]
        }
    }

    //方向键翻页==============
    makeKeyboardNav = function (p) {
        var tid = p[0].split("=")[1];
        var total_page = p[1];
        var cur = p[2];
        document.onkeyup = function (e) {
            var keyCode = e.keyCode;
            if (keyCode === 37) { // LeftArrow
                if (cur - 1 >= 1) { window.location.href = `tid=${tid}&page=${cur - 1}.html`; }
            } else if (keyCode === 39) { // RightArrow
                if (cur + 1 <= total_page) { window.location.href = `tid=${tid}&page=${cur + 1}.html`; }
            }
        }
    }

}//be