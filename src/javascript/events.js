module.exports = {
    language_load:function(){
        return new CustomEvent("language_loaded", {
            detail: {
                language: graviton.getUILanguage()
            }
        });
    },
    tabCreated:function(tabElement){
        return new CustomEvent("tab_created", {
            detail: {
                tab: tabElement
            }
        });
    },
    tabLoaded:function(tabElement){
        return new CustomEvent("tab_created", {
            detail: {
                tab: tabElement
            }
        });
    },
    tabClosed:function(tabElement){
        return new CustomEvent("tab_closed", {
            detail: {
                tab: tabElement
            }
        });
    },
    screenLoaded:function(screenID){
        return new CustomEvent("screen_loaded", {
            detail: {
                screen: screenID
            }
        });
    },
    newRecentProject: function(dir){
        return new CustomEvent("new_recent_project", {
            detail: {
                name: path.basename(dir),
                path: dir
            }
        });
    },
    splitScreen: function(){
        return new CustomEvent("split_screen", {
            detail: {
                screen:current_screen
            }
        });
    },
    closedScreen: function(screen){
        return new CustomEvent("closed_screen", {
            detail: {
                screen:screen
            }
        });
    },
    tabReorganized: function(data){
        return new CustomEvent("tab_reorganized", {
            detail: data
        });
    }
    
    
}