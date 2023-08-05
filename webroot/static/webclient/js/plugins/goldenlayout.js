/*
 *
 * Golden Layout plugin
 *
 */

var RavensGleaning = {
    html: function(str, mushLog = false) {
        function colorIndexToHtml(bold, color) {
            //console.log("Color for " + color);
            if(color < 8 && bold) {
                //console.log("\t is bold");
                color += 8;
            }
            //console.log("\t" + colorTable[color]);
            return colorTable[color];
        }

        function updateState(state, command) {
            if(!Object.keys(state).includes('foreground')) {
                state.foreground = 7;
            }

            if(!Object.keys(state).includes('background')) {
                state.background = 0;
            }

            //console.log('Command: ' + command);
            if(command.substr(-1) == "m") {
                var parts = command.substr(0, command.length - 1).split(";");
                for(var i = 0; i < parts.length; ++i) {
                    var num = parseInt(parts[i]);
                    //console.log(num);
                    // Reset
                    if(num == 0) {
                        state = { foreground: 7, background: 0 };
                    } else if(num == 1) {
                        state.bold = true;
                    } else if(num == 4) {
                        state.underscore = true;
                    } else if(num == 5) {
                        state.blink = true;
                    } else if(num == 7) {
                        state.reverse = true;
                        // 16 color FG
                    } else if(num >= 30 && num <= 37) {
                        //console.log("set fg to " + num);
                        state.foreground = num - 30;
                        // 16 color BG
                    } else if(num >= 40 && num <= 47) {
                        //console.log("set bg to " + num);
                        state.background = num - 40;
                        // Extended FG color
                    } else if(num == 38) {
                        i++;
                        // 256 color
                        if(parseInt(parts[i]) == 5) {
                            i++;
                            state.foreground = parseInt(parts[i]);
                            //console.log("set fg to " + state.foreground);
                        }
                        // Extended BG color
                    } else if(num == 48) {
                        i++;
                        // 256 color
                        if(parseInt(parts[i]) == 5) {
                            i++;
                            state.background = parseInt(parts[i]);
                            //console.log("set bg to " + state.background);
                        }
                    }
                }
            }

            //console.log(state);
            return state;
        }

        function isStateReset(state) {
            return !state.bold &&
                !state.underscore &&
                !state.blink &&
                state.foreground == 7 &&
                state.background == 0;
        }

        function htmlForState(state) {
            var ret = "";
            ret += '<span style="';
            if(state.bold) {
                ret += "font-weight:bold;";
            }
            if(state.underscore) {
                ret += "text-decoration:underline;";
            }
            if(state.blink) {
                ret += "text-decoration:blink;";
            }

            var fg = colorIndexToHtml(state.bold, state.foreground);
            var bg = colorIndexToHtml(false, state.background);

            if(state.reverse) {
                ret += "color:" + bg + ";";
                ret += "background-color:" + fg + ";";
            } else {
                ret += "color:" + fg + ";";
                ret += "background-color:" + bg + ";";
            }
            ret += '">';

            return ret;
        }

        const colorTable = {
            0: "#000000",
            1: "#800000",
            2: "#008000",
            3: "#808000",
            4: "#000080",
            5: "#800080",
            6: "#008080",
            7: "#c0c0c0",
            8: "#808080",
            9: "#ff0000",
            10: "#00ff00",
            11: "#ffff00",
            12: "#0000ff",
            13: "#ff00ff",
            14: "#00ffff",
            15: "#ffffff",
            16: "#000000",
            17: "#00005f",
            18: "#000087",
            19: "#0000af",
            20: "#0000d7",
            21: "#0000ff",
            22: "#005f00",
            23: "#005f5f",
            24: "#005f87",
            25: "#005faf",
            26: "#005fd7",
            27: "#005fff",
            28: "#008700",
            29: "#00875f",
            30: "#008787",
            31: "#0087af",
            32: "#0087d7",
            33: "#0087ff",
            34: "#00af00",
            35: "#00af5f",
            36: "#00af87",
            37: "#00afaf",
            38: "#00afd7",
            39: "#00afff",
            40: "#00d700",
            41: "#00d75f",
            42: "#00d787",
            43: "#00d7af",
            44: "#00d7d7",
            45: "#00d7ff",
            46: "#00ff00",
            47: "#00ff5f",
            48: "#00ff87",
            49: "#00ffaf",
            50: "#00ffd7",
            51: "#00ffff",
            52: "#5f0000",
            53: "#5f005f",
            54: "#5f0087",
            55: "#5f00af",
            56: "#5f00d7",
            57: "#5f00ff",
            58: "#5f5f00",
            59: "#5f5f5f",
            60: "#5f5f87",
            61: "#5f5faf",
            62: "#5f5fd7",
            63: "#5f5fff",
            64: "#5f8700",
            65: "#5f875f",
            66: "#5f8787",
            67: "#5f87af",
            68: "#5f87d7",
            69: "#5f87ff",
            70: "#5faf00",
            71: "#5faf5f",
            72: "#5faf87",
            73: "#5fafaf",
            74: "#5fafd7",
            75: "#5fafff",
            76: "#5fd700",
            77: "#5fd75f",
            78: "#5fd787",
            79: "#5fd7af",
            80: "#5fd7d7",
            81: "#5fd7ff",
            82: "#5fff00",
            83: "#5fff5f",
            84: "#5fff87",
            85: "#5fffaf",
            86: "#5fffd7",
            87: "#5fffff",
            88: "#870000",
            89: "#87005f",
            90: "#870087",
            91: "#8700af",
            92: "#8700d7",
            93: "#8700ff",
            94: "#875f00",
            95: "#875f5f",
            96: "#875f87",
            97: "#875faf",
            98: "#875fd7",
            99: "#875fff",
            100: "#878700",
            101: "#87875f",
            102: "#878787",
            103: "#8787af",
            104: "#8787d7",
            105: "#8787ff",
            106: "#87af00",
            107: "#87af5f",
            108: "#87af87",
            109: "#87afaf",
            110: "#87afd7",
            111: "#87afff",
            112: "#87d700",
            113: "#87d75f",
            114: "#87d787",
            115: "#87d7af",
            116: "#87d7d7",
            117: "#87d7ff",
            118: "#87ff00",
            119: "#87ff5f",
            120: "#87ff87",
            121: "#87ffaf",
            122: "#87ffd7",
            123: "#87ffff",
            124: "#af0000",
            125: "#af005f",
            126: "#af0087",
            127: "#af00af",
            128: "#af00d7",
            129: "#af00ff",
            130: "#af5f00",
            131: "#af5f5f",
            132: "#af5f87",
            133: "#af5faf",
            134: "#af5fd7",
            135: "#af5fff",
            136: "#af8700",
            137: "#af875f",
            138: "#af8787",
            139: "#af87af",
            140: "#af87d7",
            141: "#af87ff",
            142: "#afaf00",
            143: "#afaf5f",
            144: "#afaf87",
            145: "#afafaf",
            146: "#afafd7",
            147: "#afafff",
            148: "#afd700",
            149: "#afd75f",
            150: "#afd787",
            151: "#afd7af",
            152: "#afd7d7",
            153: "#afd7ff",
            154: "#afff00",
            155: "#afff5f",
            156: "#afff87",
            157: "#afffaf",
            158: "#afffd7",
            159: "#afffff",
            160: "#d70000",
            161: "#d7005f",
            162: "#d70087",
            163: "#d700af",
            164: "#d700d7",
            165: "#d700ff",
            166: "#d75f00",
            167: "#d75f5f",
            168: "#d75f87",
            169: "#d75faf",
            170: "#d75fd7",
            171: "#d75fff",
            172: "#d78700",
            173: "#d7875f",
            174: "#d78787",
            175: "#d787af",
            176: "#d787d7",
            177: "#d787ff",
            178: "#d7af00",
            179: "#d7af5f",
            180: "#d7af87",
            181: "#d7afaf",
            182: "#d7afd7",
            183: "#d7afff",
            184: "#d7d700",
            185: "#d7d75f",
            186: "#d7d787",
            187: "#d7d7af",
            188: "#d7d7d7",
            189: "#d7d7ff",
            190: "#d7ff00",
            191: "#d7ff5f",
            192: "#d7ff87",
            193: "#d7ffaf",
            194: "#d7ffd7",
            195: "#d7ffff",
            196: "#ff0000",
            197: "#ff005f",
            198: "#ff0087",
            199: "#ff00af",
            200: "#ff00d7",
            201: "#ff00ff",
            202: "#ff5f00",
            203: "#ff5f5f",
            204: "#ff5f87",
            205: "#ff5faf",
            206: "#ff5fd7",
            207: "#ff5fff",
            208: "#ff8700",
            209: "#ff875f",
            210: "#ff8787",
            211: "#ff87af",
            212: "#ff87d7",
            213: "#ff87ff",
            214: "#ffaf00",
            215: "#ffaf5f",
            216: "#ffaf87",
            217: "#ffafaf",
            218: "#ffafd7",
            219: "#ffafff",
            220: "#ffd700",
            221: "#ffd75f",
            222: "#ffd787",
            223: "#ffd7af",
            224: "#ffd7d7",
            225: "#ffd7ff",
            226: "#ffff00",
            227: "#ffff5f",
            228: "#ffff87",
            229: "#ffffaf",
            230: "#ffffd7",
            231: "#ffffff",
            232: "#080808",
            233: "#121212",
            234: "#1c1c1c",
            235: "#262626",
            236: "#303030",
            237: "#3a3a3a",
            238: "#444444",
            239: "#4e4e4e",
            240: "#585858",
            241: "#606060",
            242: "#666666",
            243: "#767676",
            244: "#808080",
            245: "#8a8a8a",
            246: "#949494",
            247: "#9e9e9e",
            248: "#a8a8a8",
            249: "#b2b2b2",
            250: "#bcbcbc",
            251: "#c6c6c6",
            252: "#d0d0d0",
            253: "#dadada",
            254: "#e4e4e4",
            255: "#eeeeee"
        }

        var buf = str;
        var len = str.length;
        var ret = "";
        var offset = 0;
        var state = {};
        state = updateState(state, '');
        if(!mushLog) {
            ret = htmlForState(state);
        }
        do { try {
            // Read next byte
            var byte = buf.codePointAt(offset);

            // If we see ESC, get to work
            if(byte == "\u001b".codePointAt(0)) {
                // If next char is [, this is a sequence we care about
                if(offset + 1 < len && buf.codePointAt(offset + 1) == "[".codePointAt(0)) {
                    // Jump past the CSI
                    offset += 2;
                    var command = "";

                    // Read ahead until we hit something that isn't a
                    // number or a semicolon
                    do {
                        var char = String.fromCodePoint(buf.codePointAt(offset));
                        command += char;
                    } while(++offset < len && char.match(/[0-9;]/));

                    var fromReset = isStateReset(state);
                    // Process the command
                    state = updateState(state, command);
                    var toReset = isStateReset(state);
                    if(!fromReset || !mushLog) {
                        ret += "</span>";
                    }
                    if(!toReset || !mushLog) {
                        ret += htmlForState(state);
                    }
                    continue;
                }
            } else if (byte == "&".codePointAt(0)) {
                ret += "&amp;";
                ++offset;
                continue;
            } else if (byte == "<".codePointAt(0)) {
                ret  += "&lt;";
                ++offset;
                continue;
            } else if (byte == ">".codePointAt(0)) {
                ret += "&gt;";
                ++offset;
                continue;
            } else if (byte == '"'.codePointAt(0)) {
                ret += "&quot;";
                ++offset;
                continue;
            }

            // Pass through
            offset++;
            ret += String.fromCodePoint(byte);
        } catch(e) { break }
        } while(offset < len);
        var isReset = isStateReset(state);
        if(!isReset || !mushLog) {
            ret += "</span>";
        }
        return ret.toString();
    }
};
window.raven = RavensGleaning;

let goldenlayout = (function () {

    var myLayout; // The actively used GoldenLayout API object.

    var evenniaGoldenLayouts = new Map(); // key/value Map for each selectable layout.
    var activeLayoutName = "default"; // The object key of the active evenniaGoldenLayout
    var activeLayoutModified = false; // Has the active layout been modified by the user, without being saved?

    var knownTypes = ["all", "untagged", "testing"];
    var untagged = [];

    var newTabConfig = {
        title: "Untitled",
        type: "component",
        componentName: "evennia",
        tooltip: "Click and drag tabs to make new panes",
        componentState: {
            types: "all",
            updateMethod: "newlines",
        },
    };

    var newInputConfig = {
        title: "input",
        type: "component",
        componentName: "input",
        id: "inputComponent",
    };

    // helper function: only allow a function to be called once
    function once(func) {
      function _f() {
        if (!_f.isCalled) {
          _f.isCalled = true;
          _f.res = func.apply(this, arguments);
        }
        return _f.res;
      }
      _f.prototype = func.prototype;
      _f.isCalled = false;
      return _f;
    }

    // helper function:  filter vals out of array
    function filter (vals, array) {
        if( Array.isArray( vals ) && Array.isArray( array ) ) {
            let tmp = array.slice();
            vals.forEach( function (val) {
                while( tmp.indexOf(val) > -1 ) {
                    tmp.splice( tmp.indexOf(val), 1 );
                }
            });
            return tmp;
        }
        // pass along whatever we got, since our arguments aren't right.
        return array;
    }


    //
    // Calculate all knownTypes minus the "all" type,
    //     then filter out all types that have been mapped to a pane.
    var calculateUntaggedTypes = function () {
        // set initial untagged list
        untagged = filter( ["all", "untagged"], knownTypes);
        // for each .content pane
        $(".content").each( function () {
            let types = $(this).attr("types");
            if ( typeof types !== "undefined" ) {
                let typesArray = types.split(" ");
                // add our types to known types so that the onText function don't add them to untagged later
                knownTypes = Array.from(new Set([...knownTypes, ...typesArray]));
                // remove our types from the untagged array                
                untagged = filter( typesArray, untagged );
            }
        });
    }


    //
    //
    var closeRenameDropdown = function () {
        let content = $("#renamebox").parent().parent().parent().parent()[0];
        let title = $("#renameboxin").val();

        let components = myLayout.root.getItemsByType("component");

        components.forEach( function (component) {
           let element = component.tab.header.parent.element[0];
           if( (element === content) && (component.tab.isActive) ) {
               component.setTitle( title );
           }
        });

        myLayout.emit("stateChanged");
        $("#renamebox").remove();
        window.plugins["default_in"].setKeydownFocus(true);
    }


    //
    //
    var closeTypelistDropdown = function () {
        let content = $("#typelist").parent().find(".content");
        let checkboxes = $("#typelist :input");

        let types = [];
        checkboxes.each( function (idx) {
            if( $(checkboxes[idx]).prop("checked") ) {
                types.push( $(checkboxes[idx]).val() );
            }
        });

        content.attr("types", types.join(" "));
        myLayout.emit("stateChanged");

        calculateUntaggedTypes();
        $("#typelist").remove();
    }


    //
    //
    var closeUpdatelistDropdown = function () {
        let content = $("#updatelist").parent().find(".content");
        let value   = $("input[name=upmethod]:checked").val();

        content.attr("updateMethod", value );
        myLayout.emit("stateChanged");
        $("#updatelist").remove();
    }


    //
    // Handle the renameDropdown
    var renameDropdown = function (evnt) {
        let element = $(evnt.data.contentItem.element);
        let content = element.find(".content");
        let title   = evnt.data.contentItem.config.title;
        let renamebox = document.getElementById("renamebox");

        // check that no other dropdown is open
        if( document.getElementById("typelist") ) {
            closeTypelistDropdown();
        }

        if( document.getElementById("updatelist") ) {
            closeUpdatelistDropdown();
        }

        if( !renamebox ) {
            renamebox = $("<div id='renamebox'>");
            renamebox.append("<input type='textbox' id='renameboxin' value='"+title+"'>");
            renamebox.insertBefore( content );
            window.plugins["default_in"].setKeydownFocus(false);
        } else {
            closeRenameDropdown();
        }
    }


    //
    //
    var onSelectTypesClicked = function (evnt) {
        let element = $(evnt.data.contentItem.element);
        let content = element.find(".content");
        let selectedTypes = content.attr("types");
        let menu = $("<div id='typelist'>");
        let div = $("<div class='typelistsub'>");

        if( selectedTypes ) {
            selectedTypes = selectedTypes.split(" ");
        }
        knownTypes.forEach( function (itype) {
            let choice;
            if( selectedTypes && selectedTypes.includes(itype) ) {
                choice = $("<label><input type='checkbox' value='"+itype+"' checked='checked'/>"+itype+"</label>");
            } else {
                choice = $("<label><input type='checkbox' value='"+itype+"'/>"+itype+"</label>");
            }
            choice.appendTo(div);
        });
        div.appendTo(menu);

        element.prepend(menu);
    }


    //
    // Handle the typeDropdown
    var typeDropdown = function (evnt) {
        let typelist = document.getElementById("typelist");

        // check that no other dropdown is open
        if( document.getElementById("renamebox") ) {
            closeRenameDropdown();
        }

        if( document.getElementById("updatelist") ) {
            closeUpdatelistDropdown();
        }

        if( !typelist ) {
            onSelectTypesClicked(evnt);
        } else {
            closeTypelistDropdown();
        }
    }


    //
    //
    var onUpdateMethodClicked = function (evnt) {
        let element = $(evnt.data.contentItem.element);
        let content = element.find(".content");
        let updateMethod = content.attr("updateMethod");
        let nlchecked = (updateMethod === "newlines") ? "checked='checked'" : "";
        let apchecked = (updateMethod === "append")   ? "checked='checked'" : "";
        let rpchecked = (updateMethod === "replace")  ? "checked='checked'" : "";

        let menu = $("<div id='updatelist'>");
        let div = $("<div class='updatelistsub'>");

        let newlines = $("<label><input type='radio' name='upmethod' value='newlines' "+nlchecked+"/>Newlines</label>");
        let append   = $("<label><input type='radio' name='upmethod' value='append' "+apchecked+"/>Append</label>");
        let replace  = $("<label><input type='radio' name='upmethod' value='replace' "+rpchecked+"/>Replace</label>");

        newlines.appendTo(div);
        append.appendTo(div);
        replace.appendTo(div);

        div.appendTo(menu);

        element.prepend(menu);
    }


    //
    // Handle the updateDropdown
    var updateDropdown = function (evnt) {
        let updatelist = document.getElementById("updatelist");

        // check that no other dropdown is open
        if( document.getElementById("renamebox") ) {
            closeRenameDropdown();
        }

        if( document.getElementById("typelist") ) {
            closeTypelistDropdown();
        }

        if( !updatelist ) {
            onUpdateMethodClicked(evnt);
        } else {
            closeUpdatelistDropdown();
        }
    }

    //
    // ensure only one handler is set up on the parent with once
    var registerInputTabChangeHandler = once(function (tab) {
        // Set up the control to add new tabs
        let splitControl = $(
          "<span class='lm_title' style='font-size: 1.5em;width: 1em;'>+</span>"
        );

        // Handler for adding a new tab
        splitControl.click( tab, function (evnt) {
            evnt.data.header.parent.addChild( newInputConfig );
        });

        // Position it after the tab list
        $('ul.lm_tabs', tab.header.element).after(splitControl).css("position", "relative");
        tab.header.parent.on( "activeContentItemChanged", onActiveInputTabChange );
    });

    //
    // Handle when the active input tab changes
    var onActiveInputTabChange = function (tab) {
      $('.inputfield').removeClass('focused');
      $('.inputfield', tab.tab.contentItem.element).addClass('focused');
    }

    //
    // ensure only one handler is set up on the parent with once
    var registerMainTabChangeHandler = once(function (tab) {
      tab.header.parent.on( "activeContentItemChanged", onActiveMainTabChange );
    });

    //
    // Handle when the active main tab changes
    var onActiveMainTabChange = function (tab) {
        let renamebox  = document.getElementById("renamebox");
        let typelist   = document.getElementById("typelist");
        let updatelist = document.getElementById("updatelist");

        if( renamebox ) {
            closeRenameDropdown();
        }

        if( typelist ) {
            closeTypelistDropdown();
        }

        if( updatelist ) {
            closeUpdatelistDropdown();
        }
    }

    //
    // Save the GoldenLayout state to localstorage whenever it changes.
    var onStateChanged = function () {
        let components = myLayout.root.getItemsByType("component");
        components.forEach( function (component) {
            if( component.hasId("inputComponent") ) { return; } // ignore input components

            let textDiv = component.container.getElement().children(".content");
            let types = textDiv.attr("types");
            let updateMethod = textDiv.attr("updateMethod");
            component.container.extendState({ "types": types, "updateMethod": updateMethod });
        });

        // update localstorage
        localStorage.setItem( "evenniaGoldenLayoutSavedState", JSON.stringify(myLayout.toConfig()) );
        localStorage.setItem( "evenniaGoldenLayoutSavedStateName", activeLayoutName );
    }


    //
    //
    var onClearLocalstorage = function (evnt) {
        myLayout.off( "stateChanged", onStateChanged );
        localStorage.removeItem( "evenniaGoldenLayoutSavedState" );
        localStorage.removeItem( "evenniaGoldenLayoutSavedStateName" );
        location.reload();
    }


    //
    //
    var scrollAll = function () {
        let components = myLayout.root.getItemsByType("component");
        components.forEach( function (component) {
            if( component.hasId("inputComponent") ) { return; } // ignore input components

            let textDiv = component.container.getElement().children(".content");
            let scrollHeight = textDiv.prop("scrollHeight");
            let clientHeight = textDiv.prop("clientHeight");
            textDiv.scrollTop( scrollHeight - clientHeight );
        });
        myLayout.updateSize();
    }


    //
    //
    var onTabCreate = function (tab) {
        //HTML for the typeDropdown
        let renameDropdownControl = $("<span class='lm_title' style='font-size: 1.5em;width: 0.5em;'>&#9656;</span>");
        let typeDropdownControl   = $("<span class='lm_title' style='font-size: 1.0em;width: 1em;'>&#9670;</span>");
        let updateDropdownControl = $("<span class='lm_title' style='font-size: 1.0em;width: 1em;'>&#9656;</span>");
        let splitControl          = $("<span class='lm_title' style='font-size: 1.5em;width: 1em;'>+</span>");
        // track dropdowns when the associated control is clicked
        renameDropdownControl.click( tab, renameDropdown ); 

        typeDropdownControl.click( tab, typeDropdown );

        updateDropdownControl.click( tab, updateDropdown );

        // track adding a new tab
        splitControl.click( tab, function (evnt) {
            evnt.data.header.parent.addChild( newTabConfig );
        });

        // Add the typeDropdown to the header
        tab.element.prepend( renameDropdownControl );
        tab.element.append(  typeDropdownControl );
        tab.element.append(  updateDropdownControl );
        tab.element.append(  splitControl );

        if( tab.contentItem.config.componentName === "Main" ) {
            tab.element.prepend( $("#optionsbutton").clone(true).addClass("lm_title") );
        }

        registerMainTabChangeHandler(tab);
    }


    //
    //
    var onInputCreate = function (tab) {
        registerInputTabChangeHandler(tab);
    }


    //
    //
    var initComponent = function (div, container, state, defaultTypes, updateMethod) {
        // set this container"s content div types attribute
        if( state ) {
            div.attr("types", state.types);
            div.attr("updateMethod", state.updateMethod);
        } else {
            div.attr("types", defaultTypes);
            div.attr("updateMethod", updateMethod);
        }
        div.appendTo( container.getElement() );
        container.on("tab", onTabCreate);
    }


    //
    //
    var registerComponents = function (myLayout) {

        // register our component and replace the default messagewindow with the Main component
        myLayout.registerComponent( "Main", function (container, componentState) {
            let main = $("#messagewindow").addClass("content");
            initComponent(main, container, componentState, "untagged", "newlines" );
        });

        // register our input component
        myLayout.registerComponent( "input", function (container, componentState) {
            var promptfield = $("<div class='prompt'></div>");
            var formcontrol = $("<textarea type='text' class='inputfield form-control'></textarea>");
            var button = $("<button type='button' class='inputsend'>&gt;</button>");
 
            var inputfield = $("<div class='inputfieldwrapper'>")
                                .append( button )
                                .append( formcontrol );

            $("<div class='inputwrap'>")
                .append( promptfield )
                .append( inputfield )
                .appendTo( container.getElement() );

            button.bind("click", function (evnt) {
                // focus our textarea
                $( $(evnt.target).siblings(".inputfield")[0] ).focus();
                // fake a carriage return event
                var e = $.Event("keydown");
                e.which = 13;
                $( $(evnt.target).siblings(".inputfield")[0] ).trigger(e);
            });

            container.on("tab", onInputCreate);
        });

        // register the generic "evennia" component
        myLayout.registerComponent( "evennia", function (container, componentState) {
            let div = $("<div class='content'></div>");
            initComponent(div, container, componentState, "all", "newlines");
            container.on("destroy", calculateUntaggedTypes);
        });
    }


    //
    //
    var resetUI = function (newLayout) {
        var mainsub = document.getElementById("main-sub");

        // rebuild the original HTML stacking
        var messageDiv = $("#messagewindow").detach();
        messageDiv.prependTo( mainsub );

        // out with the old
        myLayout.destroy();

        // in with the new
        myLayout = new window.GoldenLayout( newLayout, mainsub );

        // re-register our main, input and generic evennia components.
        registerComponents( myLayout );

        // call all other plugins to give them a chance to registerComponents.
        for( let plugin in window.plugins ) {
            if( "onLayoutChanged" in window.plugins[plugin] ) {
                window.plugins[plugin].onLayoutChanged();
            }
        }

        // finish the setup and actually start GoldenLayout
        myLayout.init();

        // work out which types are untagged based on our pre-configured layout
        calculateUntaggedTypes();

        // Set the Event handler for when the client window changes size
        $(window).bind("resize", scrollAll);

        // Set Save State callback
        myLayout.on( "stateChanged", onStateChanged );
    }


    //
    //
    var onSwitchLayout = function (evnt) {
        // get the new layout name from the select box
        var name       = $(evnt.target).val();
        var saveButton = $(".savelayout");

        // check to see if the layout is in the list of known layouts
        if( evenniaGoldenLayouts.has(name) ) {
            var newLayout = evenniaGoldenLayouts.get(name);

            // reset the activeLayout
            activeLayoutName = name;
            activeLayoutModified = false;

            if( activeLayoutName === "default" ) {
                saveButton.prop( "disabled", true );
            } else {
                saveButton.prop( "disabled", false );
            }

            // store the newly requested layout into localStorage.
            localStorage.setItem( "evenniaGoldenLayoutSavedState", JSON.stringify(newLayout) );
            localStorage.setItem( "evenniaGoldenLayoutSavedStateName", activeLayoutName );

            // pull the trigger
            resetUI( newLayout );
        }
    }


    //
    // upload the named layout to the Evennia server as an option
    var uploadLayouts = function () {
        if( window.Evennia.isConnected() && myLayout.isInitialised ) {
            var obj = {};

            // iterate over each layout, storing the json for each into our temp obj
            for( const key of evenniaGoldenLayouts.keys() ) {
                if( key !== "default" ) {
                    obj[key] = JSON.stringify( evenniaGoldenLayouts.get(key) );
                }
            }

            // store our temp object as json out to window.options.webclientLayouts
            window.options["webclientActiveLayout"] = activeLayoutName;
            window.options["webclientLayouts"] = JSON.stringify( obj );
            window.Evennia.msg("webclient_options", [], window.options);
        }
    }



    //
    //
    var onRemoveLayout = function (evnt) {
        var name = $(evnt.target).parent().attr("id");
        var layout = $("#"+name);

        evenniaGoldenLayouts.delete(name);
        layout.remove();

        uploadLayouts();
    }


    //
    // This is a helper function for when adding items from the OptionsUI's layout listing
    var addLayoutUI = function (layoutDiv, name) {
        var div = $("<div id='"+name+"' >");

        var option = $("<input type='button' class='goldenlayout' value='"+name+"'>");
        option.on("click", onSwitchLayout);
        div.append(option);

        if( name !== "default" && name !== activeLayoutName ) {
            var remove = $("<input type='button' class='removelayout' value='X'>");
            remove.on("click", onRemoveLayout);
            div.append(remove);
        }

        layoutDiv.append(div);
    }


    //
    //
    var onSaveLayout = function () {
        // get the name from the select box
        var name = $("#layoutName").val();
        var layouts = $("#goldenlayouts");

        // make sure we have a valid name
        if( name !== "" ) {
            // Is this name new or pre-existing?
            if( !evenniaGoldenLayouts.has(name) ) {
                // this is a new name, so add a new UI item for it.
                addLayoutUI( layouts, name );
            }

            // Force Close the Options Menu so that it isn't part of the saved layout.
            window.plugins["options2"].onOpenCloseOptions();

            // store the current layout to the local list of layouts
            evenniaGoldenLayouts.set( name, myLayout.toConfig() );
            activeLayoutName = name;
            activeLayoutModified = false;

            // store the newly requested layout into localStorage.
            localStorage.setItem( "evenniaGoldenLayoutSavedState", JSON.stringify( evenniaGoldenLayouts.get(name) ) );
            localStorage.setItem( "evenniaGoldenLayoutSavedStateName", activeLayoutName );

            uploadLayouts();

            resetUI( evenniaGoldenLayouts.get(name) );
        }
    }


    //
    // Public
    //

    //
    // helper accessor for other plugins to add new known-message types
    var addKnownType = function (newtype) {
        if( knownTypes.includes(newtype) == false ) {
            knownTypes.push(newtype);
        }
    }


    //
    // Add new HTML message to an existing Div pane, while
    // honoring the pane's updateMethod and scroll state, etc.
    //
    var addMessageToPaneDiv = function (textDiv, message, kwargs) {
        let atBottom = false;
        let updateMethod = textDiv.attr("updateMethod");

        if ( updateMethod === "replace" ) {
            textDiv.html(message);
        } else if ( updateMethod === "append" ) {
            textDiv.append(message);
        } else {  // line feed
            var cls = (kwargs === undefined) || (kwargs['cls'] === undefined) ? 'out' : kwargs['cls'];
            textDiv.append("<div class='" + cls + "'>" + message + "</div>");
        }

        // Calculate the scrollback state.
        //
        // This check helps us avoid scrolling to the bottom when someone is
        // manually scrolled back, trying to read their backlog.
        // Auto-scrolling would force them to re-scroll to their previous scroll position.
        // Which, on fast updating games, destroys the utility of scrolling entirely.
        //
        //if( textDiv.scrollTop === (textDiv.scrollHeight - textDiv.offsetHeight) ) {
            atBottom = true;
        //}

        // if we are at the bottom of the window already, scroll to display the new content
        if( atBottom ) {
            let scrollHeight = textDiv.prop("scrollHeight");
            let clientHeight = textDiv.prop("clientHeight");
            textDiv.scrollTop( scrollHeight - clientHeight );
        }
    }


    //
    // returns an array of pane divs that the given message should be sent to
    //
    var routeMessage = function (args, kwargs) {
        // If the message is not itself tagged, we"ll assume it
        // should go into any panes with "all" and "untagged" set
        var divArray = [];
        var msgtype = "untagged";

        if ( kwargs && "type" in kwargs ) {
            msgtype = kwargs["type"];
            if ( ! knownTypes.includes(msgtype) ) {
                // this is a new output type that can be mapped to panes
                knownTypes.push(msgtype);
                untagged.push(msgtype);
            }
        }

        let components = myLayout.root.getItemsByType("component");
        components.forEach( function (component) {
            if( component.hasId("inputComponent") ) { return; } // ignore input components

            let destDiv = component.container.getElement().children(".content");
            let attrTypes = destDiv.attr("types");
            let paneTypes = attrTypes ? attrTypes.split(" ") : [];

            // is this message type listed in this pane"s types (or is this pane catching "all")
            if( paneTypes.includes(msgtype) || paneTypes.includes("all") ) {
                divArray.push(destDiv);
            }

            // is this pane catching "upmapped" messages?
            // And is this message type listed in the untagged types array?
            if( paneTypes.includes("untagged") && untagged.includes(msgtype) ) {
                divArray.push(destDiv);
            }
        });

        return divArray;
    }


    //
    //
    var onGotOptions = function (args, kwargs) {
        // Reset the UI if the JSON layout sent from the server doesn't match the client's current JSON
        if( "webclientLayouts" in kwargs ) {
            var layouts = JSON.parse( kwargs["webclientLayouts"] );

            // deserialize key/layout pairs into evenniaGoldenLayouts
            for( var key in layouts ) {
                if( key !== "default" && layouts.hasOwnProperty(key) ) { // codacy.com guard-rail
                    evenniaGoldenLayouts.set( key, JSON.parse(layouts[key]) );
                }
            }
        }
    }


    //
    //
    var onOptionsUI = function (parentdiv) {
        var layoutName = $("<input id='layoutName' type='text' class='layoutName'>");
        var saveButton = $("<input type='button' class='savelayout' value='Close Options and Save'>");
        var layoutDiv  = $("<div id='goldenlayouts'>");

        if( activeLayoutName === "default" ) {
            saveButton.prop( "disabled", true );
        }

        for (const name of evenniaGoldenLayouts.keys() ) {
            addLayoutUI(layoutDiv, name);
        }

        // currently active layout
        layoutName.val( activeLayoutName );
        layoutName.on("keydown", function (evnt) {
            var name = $(evnt.target).val();
            if( name === "default" || name === "" ) {
                saveButton.prop( "disabled", true );
            } else {
                saveButton.prop( "disabled", false );
            }
        });

        // Layout selection on-change callback
        saveButton.on("click",  onSaveLayout);

        var saveDiv = $("<div class='goldenlayout-save-ui'>");
        saveDiv.append(layoutName);
        saveDiv.append(saveButton);

        // add the selection dialog control to our parentdiv
        parentdiv.addClass("goldenlayout-options-ui");
        parentdiv.append("<div>GoldenLayout Options:</div>");
        parentdiv.append("<div>Activate a new layout:</div>");
        parentdiv.append(layoutDiv);
        parentdiv.append("<div>Save current layout as (best if used when logged in):</div>");
        parentdiv.append(saveDiv);
    }


    //
    //
    var onText = function (args, kwargs) {
        // are any panes set to receive this text message?
        var divs = routeMessage(args, kwargs);

        var msgHandled = false;
        let rendered = window.raven.html(args[0]);
        divs.forEach( function (div) {
            // yes, so add this text message to the target div
            addMessageToPaneDiv( div, rendered, kwargs );
            msgHandled = true;
        });

        return msgHandled;
    }


    //
    //
    var postInit = function () {
        // finish the setup and actually start GoldenLayout
        myLayout.init();

        // work out which types are untagged based on our pre-configured layout
        calculateUntaggedTypes();

        // Set the Event handler for when the client window changes size
        $(window).bind("resize", scrollAll);

        // Set Save State callback
        myLayout.on( "stateChanged", onStateChanged );
    }


    //
    // required Init
    var init = function (options) {
        // Set up our GoldenLayout instance built off of the default main-sub div
        var savedState = localStorage.getItem( "evenniaGoldenLayoutSavedState" );
        var activeName = localStorage.getItem( "evenniaGoldenLayoutSavedStateName" );
        var mainsub = document.getElementById("main-sub");

        // pre-load the evenniaGoldenLayouts with the hard-coded default
        evenniaGoldenLayouts.set( "default", window.goldenlayout_config );

        if( activeName !== null ) {
            activeLayoutName = activeName;
        }

        if( savedState !== null ) {
            // Overwrite the global-variable configuration from 
            //     webclient/js/plugins/goldenlayout_default_config.js
            //         with the version from localstorage
            evenniaGoldenLayouts.set( activeLayoutName, JSON.parse(savedState) );
        } else {
            localStorage.setItem( "evenniaGoldenLayoutSavedState", JSON.stringify( window.goldenlayout_config ) );
            localStorage.setItem( "evenniaGoldenLayoutSavedStateName", "default" );
        }

        myLayout = new window.GoldenLayout( evenniaGoldenLayouts.get(activeLayoutName), mainsub );

        $("#prompt").remove();       // remove the HTML-defined prompt div
        $("#inputcontrol").remove(); // remove the cluttered, HTML-defined input divs

        registerComponents( myLayout );
    }

    return {
        init: init,
        postInit: postInit,
        onGotOptions: onGotOptions,
        onOptionsUI: onOptionsUI,
        onText: onText,
        getGL: function () { return myLayout; },
        addKnownType: addKnownType,
        onTabCreate: onTabCreate,
        routeMessage: routeMessage,
        addMessageToPaneDiv: addMessageToPaneDiv,
    }
}());
window.plugin_handler.add("goldenlayout", goldenlayout);
