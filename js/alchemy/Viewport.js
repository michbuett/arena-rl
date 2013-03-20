Alchemy.ns('Alchemy.v');

/**
 * @class Alchemy.v.Viewport
 * @extends Alchemy.v.DomContainer
 *
 * The type representing view port
 */
Alchemy.v.Viewport = {

    cls: 'viewport',

    /**
     * @cfg {Number} maxWidth
     * the maximum width
     */
    maxWidth: undefined,

    /**
     * @cfg {Number} maxHeight
     * the maximum height
     */
    maxHeight: undefined,

    constructor: function (cfg) {
        cfg = cfg || {};
        cfg = Alchemy.mix(cfg, {
            target: Alchemy.v.DomHelper.get('body'),
            width: cfg.width || this.getMaxWidth(cfg.maxWidth),
            height: cfg.height || this.getMaxHeight(cfg.maxHeight)
        });

        _super.call(this, cfg);

        window.addEventListener('resize', this.handleWindowResize.bind(this));
        this.handleWindowResize();
    },

    /**
     * handler for the "resize" event of the window;
     * adapt the size of the viewport according to the current size of the window
     * @private
     */
    handleWindowResize: function () {
        //console.log('handleWindowResize', this.getMaxWidth(), this.getMaxHeight())
        this.setSize(this.getMaxWidth(), this.getMaxHeight());
    },

    getMaxHeight: function (max) {
        var maxHeight = max || this.maxHeight;
        var h = Alchemy.v.DomHelper.getWindowHeight();
        if (maxHeight) {
            h = Math.min(h, maxHeight);
        }
        return h;
    },

    getMaxWidth: function (max) {
        var maxWidth = max || this.maxWidth;
        var w = Alchemy.v.DomHelper.getWindowWidth()
        if (maxWidth) {
            w = Math.min(w, maxWidth);
        }
        return w;
    }
};

Alchemy.v.Viewport = Alchemy.brew({
    name: 'Viewport',
    namespace: 'Alchemy.v',
    extend: Alchemy.v.DomContainer
}, Alchemy.v.Viewport);
