(function () {
    var LoadMask = {
        cls: 'ajs-load-mask',

        message: 'Loading...',

        progress: 0,

        setProgress: function (progr) {
            this.progress = progr;
            this.updateUI();
        },

        renderContent: function (ctx) {
            ctx.push(
                '<div class="ajs-msk-ctnt">',
                    '<div class="ajs-mask-progr">', this.progress, '%</div>',
                    '<span class="ajs-mask-message">', this.message, '</span>',
                '</div>'
            );
            return ctx;
        }
    };

    Alchemy.brew({
        name: 'LoadMask',
        ns: 'Alchemy.v',
        extend: Alchemy.v.DomElement
    }, LoadMask);
})();